use std::ops::Deref;
use std::collections::HashMap;

use tonic::{Request, Response};

use crate::{config::MappingStatusStrategy, mapping::HandlerMapping};
use super::{Server, start_handler_thread, get_handler_summary};
use generated_types::{GetDirectoryStatusRequest, GetDirectoryStatusResponse, HandlerSummary, RegisterToDirectoryRequest, RegisterToDirectoryResponse, StartHandlerRequest, StartHandlerResponse, StopHandlerRequest, StopHandlerResponse, handler_summary, inter_process_server::InterProcess};

#[tonic::async_trait]
impl InterProcess for Server {
    async fn register_to_directory(&self, request:Request<RegisterToDirectoryRequest>) ->
    Result<Response<RegisterToDirectoryResponse>,tonic::Status> {
        let request = request.into_inner();
        let mapping = self.mapping.write().await;
        let request_directory_path = request.directory_path.as_str();
        match mapping.directory_mapping.get(request_directory_path) {
            Some(handler_mapping) => {
                let mut message = String::from("Directory already handled by handler - ");
                message.push_str(handler_mapping.handler_type_name.as_str());
                Ok(Response::new(RegisterToDirectoryResponse {
                    message
                }))
            }
            None => {
                // Check if requested directory is a child of any handled directory
                for directory_path in mapping.directory_mapping.keys() {
                    if request_directory_path.contains(directory_path) {
                        let mut message = "Couldn't register\nDirectory is a child of handled directory - ".to_string();
                        message.push_str(directory_path); 
                        return Ok(Response::new(RegisterToDirectoryResponse {
                            message,
                        }))
                    }
                    else if directory_path.contains(request_directory_path) {
                        let mut message = "Couldn't register\nDirectory is a parent of requested directory - ".to_string();
                        message.push_str(directory_path); 
                        return Ok(Response::new(RegisterToDirectoryResponse {
                            message,
                        }))
                    }
                }
                let handlers_json = self.handlers_json.clone();
                start_handler_thread(
                    mapping, handlers_json, 
                    request.directory_path, request.handler_type_name, request.handler_config_path
                );
                let _result = self.save_mapping().await;
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
        let mapping = mapping.deref();
        
        let directory_path = request.directory_path.as_str();
        let mut directory_states_map: HashMap<String, HandlerSummary> = HashMap::new();
        
        match mapping.directory_mapping.get(directory_path) {
            Some(handler_mapping) => {
                let state = get_handler_summary(handler_mapping);
                directory_states_map.insert(directory_path.to_string(), state);
                Ok(Response::new(GetDirectoryStatusResponse {
                    directory_states_map
                }))
            }
            None => {
                if directory_path.is_empty() { // If empty - All directories are requested
                    for (directory_path, handler_mapping) in mapping.directory_mapping.iter() {
                        let state = get_handler_summary(handler_mapping);
                        directory_states_map.insert(directory_path.to_owned(), state);
                    }
                }
                Ok(Response::new(GetDirectoryStatusResponse {
                    directory_states_map
                }))
            }
        }
    }

    async fn start_handler(&self,request:Request<StartHandlerRequest>,)->Result<Response<StartHandlerResponse>,tonic::Status> {
        let request = request.into_inner();
        let mapping = self.mapping.write().await;
                                
        match mapping.directory_mapping.get(&request.directory_path) {
            Some(handler_mapping) => {
                match handler_mapping.status() {
                    handler_summary::Status::Dead => {
                        let handler_type_name = handler_mapping.handler_type_name.clone();
                        let handler_config_path = handler_mapping.handler_config_path.clone();
                        let handlers_json = self.handlers_json.clone();
                        start_handler_thread(
                            mapping, handlers_json, 
                            request.directory_path, handler_type_name, handler_config_path
                        );
                        Ok(Response::new(StartHandlerResponse {
                            message: String::from("Handler started")
                        }))
                    }
                    handler_summary::Status::Live => {
                        Ok(Response::new(StartHandlerResponse {
                            message: String::from("Handler already up")
                        }))
                    }
                }
            }
            None => {
                Ok(Response::new(StartHandlerResponse {
                    message: String::from("Directory unhandled"),
                }))
            }
        }
    }

    async fn stop_handler(&self,request:Request<StopHandlerRequest>,)->Result<Response<StopHandlerResponse>,tonic::Status> {
        let request = request.into_inner();
        let mut mapping = self.mapping.write().await;
        
        match mapping.directory_mapping.get(&request.directory_path) {
            Some(handler_mapping) => {
                let handler_type_name = handler_mapping.handler_type_name.clone();
                let handler_config_path = handler_mapping.handler_config_path.clone();

                match handler_mapping.status() {
                    handler_summary::Status::Dead => {
                        let mut message = String::from("Handler already stopped");
                        if request.remove {
                            mapping.directory_mapping.remove(&request.directory_path);
                            message.push_str(" & removed");
                            drop(mapping);
                            let _result = self.save_mapping().await;
                        }
                        else {
                            mapping.directory_mapping.insert(request.directory_path, HandlerMapping {
                                handler_thread_tx: Option::None,
                                handler_type_name,
                                handler_config_path,
                            });
                        }
                        Ok(Response::new(StopHandlerResponse {
                            message,
                        }))
                    }
                    handler_summary::Status::Live => {
                        match handler_mapping.stop_handler_thread().await {
                            Ok(mut message) => {
                                if request.remove {
                                    mapping.directory_mapping.remove(&request.directory_path);
                                    message.push_str(" & removed");
                                    drop(mapping);
                                    let _result = self.save_mapping().await;
                                }
                                else {
                                    mapping.directory_mapping.insert(request.directory_path, HandlerMapping {
                                        handler_thread_tx: Option::None,
                                        handler_type_name,
                                        handler_config_path,
                                    });
                                }
                                Ok(Response::new(StopHandlerResponse {
                                    message,
                                }))
                            }
                            Err(message) => {
                                Ok(Response::new(StopHandlerResponse {
                                    message,
                                }))
                            }
                        }
                    }
                }
            }
            None => {
                Ok(Response::new(StopHandlerResponse {
                    message: String::from("Directory unhandled"),
                }))
            }
        }
    }
}