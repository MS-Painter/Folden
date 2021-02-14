use std::ops::Deref;
use std::collections::HashMap;

use tonic::{Request, Response};

use crate::mapping::HandlerMapping;
use super::{Server, spawn_handler, spawn_handler_thread, get_handler_summary};
use generated_types::{
    GetDirectoryStatusRequest, GetDirectoryStatusResponse, RegisterToDirectoryRequest, StartHandlerRequest, StopHandlerRequest,  
    HandlerStateResponse, HandlerStatesMapResponse, HandlerStatus, HandlerSummary, inter_process_server::InterProcess};

#[tonic::async_trait]
impl InterProcess for Server {
    async fn register_to_directory(&self, request:Request<RegisterToDirectoryRequest>) ->
    Result<Response<HandlerStateResponse>,tonic::Status> {
        let request = request.into_inner();
        let mapping = self.mapping.write().await;
        let request_directory_path = request.directory_path.as_str();
        match mapping.directory_mapping.get(request_directory_path) {
            Some(handler_mapping) => {
                let mut message = String::from("Directory already handled by handler - ");
                message.push_str(handler_mapping.handler_type_name.as_str());
                Ok(Response::new(HandlerStateResponse {
                    message,
                    state: HandlerStatus::Live as i32,
                }))
            }
            None => {
                // Check if requested directory is a child of any handled directory
                for directory_path in mapping.directory_mapping.keys() {
                    if request_directory_path.contains(directory_path) {
                        let mut message = "Couldn't register\nDirectory is a child of handled directory - ".to_string();
                        message.push_str(directory_path); 
                        return Ok(Response::new(HandlerStateResponse {
                            message,
                            state: HandlerStatus::Dead as i32,
                        }))
                    }
                    else if directory_path.contains(request_directory_path) {
                        let mut message = "Couldn't register\nDirectory is a parent of requested directory - ".to_string();
                        message.push_str(directory_path); 
                        return Ok(Response::new(HandlerStateResponse {
                            message,
                            state: HandlerStatus::Dead as i32,
                        }))
                    }
                }
                let handlers_json = self.handlers_json.clone();
                spawn_handler_thread(
                    mapping, handlers_json, 
                    request.directory_path, request.handler_type_name, request.handler_config_path
                );
                let _result = self.save_mapping().await;
                Ok(Response::new(HandlerStateResponse {
                    message: "".to_string(),
                    state: HandlerStatus::Live as i32,
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

    async fn start_handler(&self,request:Request<StartHandlerRequest>,)->Result<Response<HandlerStatesMapResponse>,tonic::Status> {
        let request = request.into_inner();
        let mapping = self.mapping.write().await;
        let directory_path = request.directory_path.as_str();
        let mut states_map: HashMap<String, HandlerStateResponse> = HashMap::new();
                                
        match mapping.clone().directory_mapping.get(directory_path) {
            Some(handler_mapping) => {
                let response = spawn_handler(mapping, self.handlers_json.clone(), directory_path, handler_mapping);
                states_map.insert(directory_path.to_owned(), response);
                Ok(Response::new(HandlerStatesMapResponse {
                    states_map,
                }))
            }
            None => {
                states_map.insert(directory_path.to_owned(), HandlerStateResponse {
                    state: HandlerStatus::Dead as i32,
                    message: String::from("Directory unhandled"),
                });
                Ok(Response::new(HandlerStatesMapResponse {
                    states_map,
                }))
            }
        }
    }

    async fn stop_handler(&self,request:Request<StopHandlerRequest>,)->Result<Response<HandlerStatesMapResponse>,tonic::Status> {
        let request = request.into_inner();
        let mut mapping = self.mapping.write().await;
        let directory_path = request.directory_path.as_str();
        let mut states_map: HashMap<String, HandlerStateResponse> = HashMap::new();
        
        match mapping.directory_mapping.get(&request.directory_path) {
            Some(handler_mapping) => {
                let handler_type_name = handler_mapping.handler_type_name.clone();
                let handler_config_path = handler_mapping.handler_config_path.clone();

                match handler_mapping.status() {
                    HandlerStatus::Dead => {
                        let mut message = String::from("Handler already stopped");
                        if request.remove {
                            mapping.directory_mapping.remove(&request.directory_path);
                            message.push_str(" & removed");
                            drop(mapping);
                            let _result = self.save_mapping().await;
                        }
                        else {
                            mapping.directory_mapping.insert(directory_path.to_owned(), HandlerMapping {
                                handler_thread_tx: Option::None,
                                handler_type_name,
                                handler_config_path,
                            });
                        }
                        states_map.insert(directory_path.to_owned(), HandlerStateResponse {
                            state: HandlerStatus::Dead as i32,
                            message,
                        });
                        Ok(Response::new(HandlerStatesMapResponse {
                            states_map,
                        }))
                    }
                    HandlerStatus::Live => {
                        match handler_mapping.stop_handler_thread().await {
                            Ok(mut message) => {
                                if request.remove {
                                    mapping.directory_mapping.remove(&request.directory_path);
                                    message.push_str(" & removed");
                                    drop(mapping);
                                    let _result = self.save_mapping().await;
                                }
                                else {
                                    mapping.directory_mapping.insert(directory_path.to_owned(), HandlerMapping {
                                        handler_thread_tx: Option::None,
                                        handler_type_name,
                                        handler_config_path,
                                    });
                                }
                                states_map.insert(directory_path.to_owned(), HandlerStateResponse {
                                    state: HandlerStatus::Dead as i32,
                                    message,
                                });
                                Ok(Response::new(HandlerStatesMapResponse {
                                    states_map,
                                }))
                            }
                            Err(message) => {
                                states_map.insert(directory_path.to_owned(), HandlerStateResponse {
                                    state: HandlerStatus::Live as i32,
                                    message,
                                });
                                Ok(Response::new(HandlerStatesMapResponse {
                                    states_map,
                                }))
                            }
                        }
                    }
                }
            }
            None => {
                states_map.insert(directory_path.to_owned(), HandlerStateResponse {
                    state: HandlerStatus::Dead as i32,
                    message: String::from("Directory unhandled"),
                });
                Ok(Response::new(HandlerStatesMapResponse {
                    states_map,
                }))
            }
        }
    }
}