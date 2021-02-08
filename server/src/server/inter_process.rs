use tokio::sync::mpsc;
use tonic::{Request, Response};

use crate::mapping::HandlerMapping;
use super::{Server, start_handler_thread};
use generated_types::{
    GetDirectoryStatusRequest, GetDirectoryStatusResponse, 
    RegisterToDirectoryRequest, RegisterToDirectoryResponse, 
    StartHandlerRequest, StartHandlerResponse, 
    StopHandlerRequest, StopHandlerResponse, 
    HandlerChannelMessage, inter_process_server::InterProcess};



#[tonic::async_trait]
impl InterProcess for Server {
    async fn register_to_directory(&self, request:Request<RegisterToDirectoryRequest>) ->
    Result<Response<RegisterToDirectoryResponse>,tonic::Status> {
        let request = request.into_inner();
        let mapping = self.mapping.read().await;
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
                drop(mapping); // Free lock here instead of scope exit
                let mapping = self.mapping.write().await;
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
        
        match mapping.directory_mapping.get(request.directory_path.as_str()) {
            Some(handler_mapping) => {
                let handler_type_name = handler_mapping.handler_type_name.clone();
                drop(mapping); // Free lock here instead of scope exit
                let mapping = self.mapping.write().await;
                let handler_mapping = mapping.directory_mapping.get(&request.directory_path).unwrap(); 

                let mut message = String::from("Handler - ");
                message.push_str(handler_type_name.as_str());
                
                match handler_mapping.handler_thread_tx.clone() {
                    Some(mut handler_thread_tx) => {
                        match handler_thread_tx.try_send(HandlerChannelMessage::Ping) {
                            Ok(_) => {
                                message.push_str(" - Alive");
                            }
                            Err(err) => {
                                match err {
                                    mpsc::error::TrySendError::Full(_) => {
                                        message.push_str(" - Alive");
                                    }
                                    mpsc::error::TrySendError::Closed(_) => {
                                        message.push_str(" - Dead");
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        message.push_str(" - Dead");
                    }
                }
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

    async fn start_handler(&self,request:Request<StartHandlerRequest>,)->Result<Response<StartHandlerResponse>,tonic::Status> {
        let request = request.into_inner();
        let mapping = self.mapping.read().await;
        
        match mapping.directory_mapping.get(&request.directory_path) {
            Some(_handler_mapping) => {
                drop(mapping); // Free lock here instead of scope exit
                let mapping = self.mapping.write().await;
                match mapping.directory_mapping.get(&request.directory_path) {
                    Some(_handler_mapping) => {
                        drop(mapping); // Free lock here instead of scope exit
                        let mut message = String::new();
                        let mapping = self.mapping.write().await;
                        let handler_mapping = mapping.directory_mapping.get(&request.directory_path).unwrap(); 
                        
                        match handler_mapping.handler_thread_tx.clone() {
                            Some(mut handler_thread_tx) => {
                                match handler_thread_tx.try_send(HandlerChannelMessage::Ping) {
                                    Ok(_) => {
                                        message = String::from("Handler already up");
                                    }
                                    Err(err) => {
                                        match err {
                                            mpsc::error::TrySendError::Full(_) => {
                                                message = String::from("Handler already up");
                                            }
                                            mpsc::error::TrySendError::Closed(_) => {
                                                let handler_type_name = handler_mapping.handler_type_name.clone();
                                                let handler_config_path = handler_mapping.handler_config_path.clone();
                                                let handlers_json = self.handlers_json.clone();
                                                start_handler_thread(
                                                    mapping, handlers_json, 
                                                    request.directory_path, handler_type_name, handler_config_path
                                                );
                                                message = String::from("Handler started");
                                            }
                                        }
                                    }
                                }
                            }
                            None => {
                                let handler_type_name = handler_mapping.handler_type_name.clone();
                                let handler_config_path = handler_mapping.handler_config_path.clone();
                                let handlers_json = self.handlers_json.clone();
                                start_handler_thread(
                                    mapping, handlers_json, 
                                    request.directory_path, handler_type_name, handler_config_path
                                );
                                message = String::from("Handler started");
                            }
                        }
                        Ok(Response::new(StartHandlerResponse {
                            message
                        }))
                    }
                    None => {
                        Ok(Response::new(StartHandlerResponse {
                            message: "Handler not found".to_string(),
                        }))
                    }
                }
            }
            None => {
                Ok(Response::new(StartHandlerResponse {
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
                        match handler_mapping.handler_thread_tx.clone() {
                            Some(mut handler_thread_tx) => {
                                let handler_type_name = handler_mapping.handler_type_name.clone();
                                let handler_config_path = handler_mapping.handler_config_path.clone();
                                match handler_thread_tx.send(HandlerChannelMessage::Terminate).await {
                                    Ok(_) => {
                                        let mut message = String::from("Handler stopped"); 
                                        if request.is_handler_to_be_removed {
                                            mapping.directory_mapping.remove(&request.directory_path);
                                            message.push_str(" & removed");
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
                                    Err(err) => {
                                        let mut message = String::new();
                                        if request.is_handler_to_be_removed {
                                            mapping.directory_mapping.remove(&request.directory_path);
                                            message = String::from("Handler removed");
                                        }
                                        else {
                                            message = String::from("Failed to stop handler\nError: ");
                                            message.push_str(err.to_string().as_str());
                                        }
                                        Ok(Response::new(StopHandlerResponse {
                                            message
                                        }))
                                    }
                                }
                            }
                            None => {
                                Ok(Response::new(StopHandlerResponse {
                                    message: String::from("")
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