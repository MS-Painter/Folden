use std::collections::HashMap;

use tonic::{Request, Response};

use super::Server;
use super::TraceHandlerStream;
use crate::handler_mapping::HandlerMapping;
use generated_types::{HandlerStateResponse, HandlerStatesMapResponse, HandlerSummary, HandlerSummaryMapResponse, handler_service_server::HandlerService};

#[tonic::async_trait]
impl HandlerService for Server {
    type TraceHandlerStream = TraceHandlerStream;

    #[tracing::instrument]
    async fn register_to_directory(&self, request:Request<generated_types::RegisterToDirectoryRequest>) -> Result<Response<HandlerStateResponse>,tonic::Status> {
        tracing::info!("Registering handler to directory");
        let request = request.into_inner();
        let mut mapping = self.mapping.write().await;
        let request_directory_path = request.directory_path.as_str();
        if mapping.directory_mapping.get(request_directory_path).is_some() {
            Ok(Response::new(HandlerStateResponse {
                is_alive: true,
                message: String::from("Directory already handled by handler"),
            }))
        }
        else {
            // Check if requested directory is a child / parent of any handled directory
            for directory_path in mapping.directory_mapping.keys() {
                if request_directory_path.contains(directory_path) {
                    return Ok(Response::new(HandlerStateResponse {
                        is_alive: false,
                        message: format!("Couldn't register\nDirectory is a child of handled directory - {}", directory_path),
                    }))
                }
                else if directory_path.contains(request_directory_path) {
                    return Ok(Response::new(HandlerStateResponse {
                        is_alive: false,
                        message: format!("Couldn't register\nDirectory is a parent of requested directory - {}", directory_path),
                    }))
                }
            }
            let mut handler_mapping = HandlerMapping::new(request.handler_config_path, request.is_auto_startup, String::new());
            if request.is_start_on_register {
                if self.is_concurrent_handlers_limit_reached(&mapping) {
                    mapping.directory_mapping.insert(request.directory_path, handler_mapping);
                    let _result = mapping.save(&self.config.mapping_state_path);
                    return Ok(Response::new(HandlerStateResponse {
                        is_alive: false,
                        message: format!("Registered handler without starting - Reached concurrent live handler limit ({})", self.config.concurrent_threads_limit),
                    }));
                }
                let trace_tx = self.handlers_trace_tx.clone();
                match mapping.spawn_handler_thread(request.directory_path, &mut handler_mapping, trace_tx) {
                    Ok(_) => {
                        let _result = mapping.save(&self.config.mapping_state_path);
                        Ok(Response::new(HandlerStateResponse {
                            is_alive: true,
                            message: String::from("Registered and started handler"),
                        }))
                    }
                    Err(err) => Ok(Response::new(HandlerStateResponse {
                        is_alive: false,
                        message: format!("Failed to register and start handler.\nError: {}", err),
                    }))
                }
            }
            else {
                mapping.directory_mapping.insert(request.directory_path, handler_mapping);
                let _result = mapping.save(&self.config.mapping_state_path);
                Ok(Response::new(HandlerStateResponse {
                    is_alive: false,
                    message: String::from("Registered handler"),
                }))
            }
        }
    }

    #[tracing::instrument]
    async fn get_directory_status(&self, request:Request<generated_types::GetDirectoryStatusRequest>) -> Result<Response<HandlerSummaryMapResponse>,tonic::Status> {
        tracing::info!("Getting directory status");
        let request = request.into_inner();
        let mapping = &*self.mapping.read().await;
        let directory_path = request.directory_path.as_str();
        let mut summary_map: HashMap<String, HandlerSummary> = HashMap::new();
        
        match mapping.directory_mapping.get(directory_path) {
            Some(handler_mapping) => {
                let state = handler_mapping.summary();
                summary_map.insert(directory_path.to_string(), state);
                Ok(Response::new(HandlerSummaryMapResponse {
                    summary_map
                }))
            }
            None => {
                // If empty - All directories are requested
                if directory_path.is_empty() {
                    for (directory_path, handler_mapping) in mapping.directory_mapping.iter() {
                        summary_map.insert(directory_path.to_owned(), handler_mapping.summary());
                    }
                }
                Ok(Response::new(HandlerSummaryMapResponse {
                    summary_map
                }))
            }
        }
    }

    #[tracing::instrument]
    async fn start_handler(&self,request:Request<generated_types::StartHandlerRequest>,)->Result<Response<HandlerStatesMapResponse>,tonic::Status> {
        tracing::info!("Starting handler");
        let request = request.into_inner();
        let mut mapping = self.mapping.write().await;
        let directory_path = request.directory_path.as_str();
        let mut states_map: HashMap<String, HandlerStateResponse> = HashMap::new();
                                
        match mapping.clone().directory_mapping.get_mut(directory_path) {
            Some(handler_mapping) => {
                if !handler_mapping.is_alive() && self.is_concurrent_handlers_limit_reached(&mapping) {
                    return Err(tonic::Status::failed_precondition(
                        format!("Aborted start handler - Reached concurrent live handler limit ({})", self.config.concurrent_threads_limit)));
                }
                let trace_tx = self.handlers_trace_tx.clone();
                let response = mapping.start_handler(directory_path, handler_mapping, trace_tx);
                states_map.insert(directory_path.to_owned(), response);
                Ok(Response::new(HandlerStatesMapResponse {
                    states_map,
                }))
            }
            None => {
                // If empty - All directories are requested
                if request.directory_path.is_empty() {
                    if mapping.directory_mapping.len() > self.config.concurrent_threads_limit.into() {
                        return Err(tonic::Status::failed_precondition(
                            format!("Aborted start handlers - Would pass concurrent live handler limit ({})\nCurrently live: {}", 
                            self.config.concurrent_threads_limit, mapping.get_live_handlers().count())));
                    }
                    for (directory_path, handler_mapping) in mapping.clone().directory_mapping.iter_mut() {
                        let trace_tx = self.handlers_trace_tx.clone();
                        let response = mapping.start_handler(directory_path, handler_mapping, trace_tx);
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

    #[tracing::instrument]
    async fn stop_handler(&self,request:Request<generated_types::StopHandlerRequest>,)->Result<Response<HandlerStatesMapResponse>,tonic::Status> {
        tracing::info!("Stopping handler");
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

    #[tracing::instrument]
    async fn modify_handler(&self,request:Request<generated_types::ModifyHandlerRequest>,)->Result<Response<()>,tonic::Status> {
        tracing::info!("Modifying handler");
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

    async fn trace_handler(&self, request: Request<generated_types::TraceHandlerRequest>) -> Result<Response<Self::TraceHandlerStream>, tonic::Status> {
        tracing::info!("Tracing directory handler");
        let request = request.into_inner();
        let mapping = self.mapping.read().await;

        if !request.directory_path.is_empty() { // If empty - All directories are requested
            match mapping.directory_mapping.get(&request.directory_path) {
                Some(handler_mapping) => {
                    if !handler_mapping.is_alive() {
                        return Err(tonic::Status::failed_precondition("Handler isn't alive to trace"));
                    }
                },
                None => return Err(tonic::Status::not_found("Directory isn't registered to handle")),
            }
        }
        else if mapping.directory_mapping.is_empty() {
            return Err(tonic::Status::not_found("No handler registered to filesystem to trace"));
        }
        else if !is_any_handler_alive(self).await {
            return Err(tonic::Status::not_found("No handler is alive to trace"));
        }

        let rx_stream = self.convert_trace_channel_reciever_to_stream();
        tracing::debug!("Handler trace receivers live: {}", self.handlers_trace_tx.receiver_count());
        return Ok(Response::new(rx_stream));
    }
}

async fn is_any_handler_alive(server: &Server) -> bool {
    let response = server.get_directory_status(Request::new(generated_types::GetDirectoryStatusRequest {
        directory_path: String::new()
    }));
    if let Ok(response) = response.await {
        let response = response.into_inner();
        return response.summary_map.iter().any(|(_dir, handler)| handler.is_alive);
    }
    false
}