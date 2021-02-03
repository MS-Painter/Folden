use std::{sync::Arc, thread};

use folder_handler::handlers_json::HandlersJson;
use tokio::sync::{RwLockWriteGuard, mpsc};
use tonic::{Request, Response};

use crate::mapping::{HandlerMapping, Mapping};
use super::Server;
use generated_types::{
    inter_process_server::InterProcess,
    StopHandlerRequest, StopHandlerResponse,
    GetDirectoryStatusRequest, GetDirectoryStatusResponse, 
    RegisterToDirectoryRequest, RegisterToDirectoryResponse, 
};

fn start_handler_thread(
    mut mapping: RwLockWriteGuard<Mapping>, handlers_json: Arc<HandlersJson>, 
    directory_path: String, handler_type_name: String, handler_config_path: String) {
    match handlers_json.get_handler_by_name(&handler_type_name) {
        Ok(_handler) => {
            let (tx, rx) = mpsc::channel::<u8>(2);
            let handler_type_name_clone = handler_type_name.clone();
            thread::spawn(move || {
                let rx = rx;
                let handlers_json = handlers_json;
                let handler = handlers_json.get_handler_by_name(&handler_type_name_clone).unwrap();
                handler.watch(rx);
            });
            
            // Insert or update the value of the current handled directory
            mapping.directory_mapping.insert(directory_path, HandlerMapping {
                handler_thread_shutdown_tx: tx,
                handler_type_name,
                handler_config_path,
            });
        },
        Err(e) => panic!(e)
    }
}

#[tonic::async_trait]
impl InterProcess for Server {
    async fn register_to_directory(&self, request:Request<RegisterToDirectoryRequest>) ->
    Result<Response<RegisterToDirectoryResponse>,tonic::Status> {
        let request = request.into_inner();
        let mapping = self.mapping.read().await;

        match mapping.directory_mapping.get(request.directory_path.as_str()) {
            Some(handler_mapping) => {
                let mut message = String::from("Directory already handled by handler - ");
                message.push_str(handler_mapping.handler_type_name.as_str());
                Ok(Response::new(RegisterToDirectoryResponse {
                    message
                }))
            }
            None => {
                drop(mapping); // Free lock here instead of scope exit
                let mapping = self.mapping.write().await;
                let handlers_json = self.handlers_json.clone();
                start_handler_thread(
                    mapping, handlers_json, 
                    request.directory_path, request.handler_type_name, request.handler_config_path
                );

                Ok(Response::new(RegisterToDirectoryResponse {
                    message: "".to_string(),
                }))
            }
        }
    }

    async fn get_directory_status(&self, request:Request<GetDirectoryStatusRequest>) ->
    Result<Response<GetDirectoryStatusResponse>,tonic::Status> {
        let request = request.into_inner();
        let mapping = self.mapping.read().await;
        
        match mapping.directory_mapping.get(request.directory_path.as_str()) {
            Some(handler_mapping) => {
                let mut message = String::from("Handler - ");
                message.push_str(handler_mapping.handler_type_name.as_str());
                Ok(Response::new(GetDirectoryStatusResponse {
                    message
                }))
            }
            None => {
                Ok(Response::new(GetDirectoryStatusResponse {
                    message: "Directory unhandled".to_string(),
                }))
            }
        }
    }

    async fn stop_handler(&self,request:Request<StopHandlerRequest>,)->Result<Response<StopHandlerResponse>,tonic::Status> {
        let request = request.into_inner();
        let mapping = self.mapping.read().await;
        
        match mapping.directory_mapping.get(&request.directory_path) {
            Some(_handler_mapping) => {
                drop(mapping); // Free lock here instead of scope exit
                let mut mapping = self.mapping.write().await;
                match mapping.directory_mapping.get(&request.directory_path) {
                    Some(handler_mapping) => {
                        let mut handler_thread_shutdown_tx = handler_mapping.handler_thread_shutdown_tx.clone();
                        match handler_thread_shutdown_tx.send(0).await {
                            Ok(_) => {
                                let mut message = String::from("Handler stopped"); 
                                if request.is_handler_to_be_removed {
                                    mapping.directory_mapping.remove(&request.directory_path);
                                    message.push_str(" & removed");
                                }
                                Ok(Response::new(StopHandlerResponse {
                                    message,
                                }))
                            }
                            Err(err) => {
                                let mut message = String::from("Failed to stop handler\nError: ");
                                message.push_str(err.to_string().as_str());
                                Ok(Response::new(StopHandlerResponse {
                                    message
                                }))
                            }
                        }
                    }
                    None => {
                        Ok(Response::new(StopHandlerResponse {
                            message: "Handler not found".to_string(),
                        }))
                    }
                }
            }
            None => {
                Ok(Response::new(StopHandlerResponse {
                    message: "Directory unhandled".to_string(),
                }))
            }
        }
    }
}