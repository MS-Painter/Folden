use std::ops::Deref;
use std::collections::HashMap;

use tonic::{Request, Response};

use super::Server;
use generated_types::{
    GetDirectoryStatusRequest, GetDirectoryStatusResponse, HandlerStateResponse, HandlerStatesMapResponse, HandlerStatus, HandlerStartupType,
    HandlerSummary, ModifyHandlerRequest, RegisterToDirectoryRequest, StartHandlerRequest, StopHandlerRequest, handler_service_server::HandlerService};

#[tonic::async_trait]
impl HandlerService for Server {
    async fn register_to_directory(&self, request:Request<RegisterToDirectoryRequest>) ->
    Result<Response<HandlerStateResponse>,tonic::Status> {
        let request = request.into_inner();
        let mut mapping = self.mapping.write().await;
        let request_directory_path = request.directory_path.as_str();
        match mapping.directory_mapping.get(request_directory_path) {
            Some(handler_mapping) => {
                Ok(Response::new(HandlerStateResponse {
                    message: String::from("Directory already handled by handler"),
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
                mapping.spawn_handler_thread(request.directory_path, request.handler_config_path);
                let _result = mapping.save(&self.config.mapping_state_path);
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
                let state = handler_mapping.summary();
                directory_states_map.insert(directory_path.to_string(), state);
                Ok(Response::new(GetDirectoryStatusResponse {
                    directory_states_map
                }))
            }
            None => {
                if directory_path.is_empty() { // If empty - All directories are requested
                    for (directory_path, handler_mapping) in mapping.directory_mapping.iter() {
                        let state = handler_mapping.summary();
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
        let mut mapping = self.mapping.write().await;
        let directory_path = request.directory_path.as_str();
        let mut states_map: HashMap<String, HandlerStateResponse> = HashMap::new();
                                
        match mapping.clone().directory_mapping.get(directory_path) {
            Some(handler_mapping) => {
                let response = mapping.start_handler(directory_path, handler_mapping);
                states_map.insert(directory_path.to_owned(), response);
                Ok(Response::new(HandlerStatesMapResponse {
                    states_map,
                }))
            }
            None => {
                if request.directory_path.is_empty() { // If empty - All directories are requested
                    for (directory_path, handler_mapping) in mapping.clone().directory_mapping.iter() {
                        let response = mapping.start_handler(directory_path, handler_mapping);
                        states_map.insert(directory_path.to_owned(), response);
                    }
                }
                else {
                    states_map.insert(directory_path.to_owned(), HandlerStateResponse {
                        state: HandlerStatus::Dead as i32,
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
        
        match mapping.clone().directory_mapping.get(&request.directory_path) {
            Some(handler_mapping) => {
                let response = mapping.stop_handler(&self.config, directory_path, handler_mapping, request.remove).await;
                states_map.insert(directory_path.to_owned(), response);
                Ok(Response::new(HandlerStatesMapResponse {
                    states_map,
                }))
            }
            None => {
                if request.directory_path.is_empty() { // If empty - All directories are requested
                    for (directory_path, handler_mapping) in mapping.clone().directory_mapping.iter() {
                        let response = mapping.stop_handler(&self.config, directory_path, handler_mapping, request.remove).await;
                        states_map.insert(directory_path.to_owned(), response);
                    }
                }
                else {
                    states_map.insert(directory_path.to_owned(), HandlerStateResponse {
                        state: HandlerStatus::Dead as i32,
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
        let inner_request = request.into_inner();
        let mut mapping = self.mapping.write().await;

        match mapping.directory_mapping.get_mut(&inner_request.directory_path) {
            Some(handler_mapping) => {
                if inner_request.startup_type != HandlerStartupType::NotProvided as i32 {
                    handler_mapping.start_on_startup = if inner_request.startup_type == HandlerStartupType::Auto as i32 {true} else {false};
                }
            }
            None => {
                if inner_request.directory_path.is_empty() { // If empty - All directories are requested
                    for handler_mapping in mapping.directory_mapping.values_mut() {
                        if inner_request.startup_type != HandlerStartupType::NotProvided as i32 {
                            handler_mapping.start_on_startup = if inner_request.startup_type == HandlerStartupType::Auto as i32 {true} else {false};
                        }
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