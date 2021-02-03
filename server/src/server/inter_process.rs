use std::thread;

use tokio::sync::oneshot;
use tonic::{Request, Response};


use crate::mapping::HandlerMapping;
use super::Server;
use generated_types::{
    inter_process_server::InterProcess, 
    GetDirectoryStatusRequest, GetDirectoryStatusResponse, 
    RegisterToDirectoryRequest, RegisterToDirectoryResponse
};

#[tonic::async_trait]
impl InterProcess for Server {
    async fn register_to_directory(&self, request:Request<RegisterToDirectoryRequest>) ->
    Result<Response<RegisterToDirectoryResponse>,tonic::Status> {
        let request = request.into_inner();
        let mapping = self.mapping.read().await;

        match mapping.directory_mapping.get(request.directory_path.as_str()) {
            Some(handler_mapping) => {
                let mut message = String::from("Directory already handled by handler - ");
                message.push_str(handler_mapping.handler_type.as_str());
                Ok(Response::new(RegisterToDirectoryResponse {
                    message
                }))
            }
            None => {
                drop(mapping); // Free lock here instead of scope exit
                let mut mapping = self.mapping.write().await;
                let handler_type_name = request.handler_type_name.clone();
                let handlers_json = self.handlers_json.clone();
                match handlers_json.get_handler_by_name(&handler_type_name) {
                    Ok(_handler) => {
                        let (tx, rx) = oneshot::channel();
                        thread::spawn(move || {
                            let rx = rx;
                            let handlers_json = handlers_json;
                            let handler = handlers_json.get_handler_by_name(&handler_type_name).unwrap();
                            handler.watch();
                        });
                        
                        mapping.directory_mapping.insert(request.directory_path, HandlerMapping {
                            handler_thread_shutdown_tx: tx,
                            handler_type: request.handler_type_name,
                            handler_config_path: request.handler_config_path,
                        });

                        println!("{:?}", mapping);
                    },
                    Err(e) => panic!(e)
                }

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
        
        println!("{:?}", mapping);

        match mapping.directory_mapping.get(request.directory_path.as_str()) {
            Some(handler_mapping) => {
                let mut message = String::from("Handler - ");
                message.push_str(handler_mapping.handler_type.as_str());
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
}