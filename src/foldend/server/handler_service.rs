use std::collections::HashMap;

use tonic::{Request, Response};

use super::Server;
use crate::mapping::HandlerMapping;
use generated_types::{
    GetDirectoryStatusRequest, GetDirectoryStatusResponse, HandlerStateResponse, HandlerStatesMapResponse,
    HandlerSummary, ModifyHandlerRequest, RegisterToDirectoryRequest, StartHandlerRequest, StopHandlerRequest, handler_service_server::HandlerService};

#[tonic::async_trait]
impl HandlerService for Server {
    async fn register_to_directory(&self, request:Request<RegisterToDirectoryRequest>) ->
    Result<Response<HandlerStateResponse>,tonic::Status> {
        let request = request.into_inner();
        let mut mapping = self.mapping.write().await;
        let request_directory_path = request.directory_path.as_str();
        match mapping.directory_mapping.get(request_directory_path) {
            Some(_handler_mapping) => {
                Ok(Response::new(HandlerStateResponse {
                    is_alive: true,
                    message: String::from("Directory already handled by handler"),
                }))
            }
            None => {
                // Check if requested directory is a child of any handled directory
                for directory_path in mapping.directory_mapping.keys() {
                    if request_directory_path.contains(directory_path) {
                        return Ok(Response::new(HandlerStateResponse {
                            is_alive: false,
                            message: format!("Couldn't register\nDirectory is a child of handled directory - {}", directory_path).to_string(),
                        }))
                    }
                    else if directory_path.contains(request_directory_path) {
                        return Ok(Response::new(HandlerStateResponse {
                            is_alive: false,
                            message: format!("Couldn't register\nDirectory is a parent of requested directory - {}", directory_path).to_string(),
                        }))
                    }
                }
                mapping.spawn_handler_thread(request.directory_path, &mut HandlerMapping {
                    watcher_tx: None,
                    handler_config_path: request.handler_config_path,
                    is_auto_startup: false,
                    description: String::new(),
                });
                let _result = mapping.save(&self.config.mapping_state_path);
                Ok(Response::new(HandlerStateResponse {
                    is_alive: true,
                    message: String::from("Registered and started handler"),
                }))
            }
        }
    }

    async fn get_directory_status(&self, request:Request<GetDirectoryStatusRequest>) ->
    Result<Response<GetDirectoryStatusResponse>,tonic::Status> {
        let request = request.into_inner();
        let mapping = &*self.mapping.read().await;
        let directory_path = request.directory_path.as_str();
        let mut directory_states_map: HashMap<String, HandlerSummary> = HashMap::new();
        
        match mapping.directory_mapping.get(directory_path) {
            Some(handler_mapping) => {
                let state = handler_mapping.summary();
                directory_states_map.insert(directory_path.to_string(), state);
                Ok(Response::new(GetDirectoryStatusResponse {
                    directory_states_map
                }))
            }
            None => {
                // If empty - All directories are requested
                if directory_path.is_empty() {
                    for (directory_path, handler_mapping) in mapping.directory_mapping.iter() {
                        directory_states_map.insert(directory_path.to_owned(), handler_mapping.summary());
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
        let mut mapping = self.mapping.write().await;
        let directory_path = request.directory_path.as_str();
        let mut states_map: HashMap<String, HandlerStateResponse> = HashMap::new();
                                
        match mapping.clone().directory_mapping.get_mut(directory_path) {
            Some(handler_mapping) => {
                let response = mapping.start_handler(directory_path, handler_mapping);
                states_map.insert(directory_path.to_owned(), response);
                Ok(Response::new(HandlerStatesMapResponse {
                    states_map,
                }))
            }
            None => {
                // If empty - All directories are requested
                if request.directory_path.is_empty() {
                    for (directory_path, handler_mapping) in mapping.clone().directory_mapping.iter_mut() {
                        let response = mapping.start_handler(directory_path, handler_mapping);
                        states_map.insert(directory_path.to_owned(), response);
                    }
                }
                else {
                    states_map.insert(directory_path.to_owned(), HandlerStateResponse {
                        is_alive: false,
                        message: String::from("Directory unhandled"),
                    });
                }
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
        
        match mapping.clone().directory_mapping.get_mut(&request.directory_path) {
            Some(handler_mapping) => {
                let response = mapping.stop_handler(&self.config, directory_path, handler_mapping, request.remove).await;
                states_map.insert(directory_path.to_owned(), response);
                Ok(Response::new(HandlerStatesMapResponse {
                    states_map,
                }))
            }
            None => {
                if request.directory_path.is_empty() { // If empty - All directories are requested
                    for (directory_path, handler_mapping) in mapping.clone().directory_mapping.iter_mut() {
                        let response = mapping.stop_handler(&self.config, directory_path, handler_mapping, request.remove).await;
                        states_map.insert(directory_path.to_owned(), response);
                    }
                }
                else {
                    states_map.insert(directory_path.to_owned(), HandlerStateResponse {
                        is_alive: false,
                        message: String::from("Directory unhandled"),
                    });
                }
                Ok(Response::new(HandlerStatesMapResponse {
                    states_map,
                }))
            }
        }
    }

    async fn modify_handler(&self,request:Request<ModifyHandlerRequest>,)->Result<Response<()>,tonic::Status> {
        let request = request.into_inner();
        let mut mapping = self.mapping.write().await;

        match mapping.directory_mapping.get_mut(&request.directory_path) {
            Some(handler_mapping) => handler_mapping.modify(&request),
            None => {
                if request.directory_path.is_empty() { // If empty - All directories are requested
                    for handler_mapping in mapping.directory_mapping.values_mut() {
                        handler_mapping.modify(&request);
                    }
                }
                else {
                    return Err(tonic::Status::not_found("Directory isn't registered to handle"));
                }
            }
        }

        match mapping.save(&self.config.mapping_state_path) {
            Ok(result) => Ok(Response::new(result)),
            Err(e) => Err(tonic::Status::unknown(format!("Failed to save modifications to mapping file.\nErr - {:?}", e)))
        }
    }
}