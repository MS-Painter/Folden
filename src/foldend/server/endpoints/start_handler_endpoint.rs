use std::collections::HashMap;

use tokio::sync::RwLockWriteGuard;

use super::handler_service_endpoint::ServiceEndpoint;
use crate::{mapping::Mapping, server::Server};
use generated_types::{HandlerStateResponse, HandlerStatesMapResponse, StartHandlerRequest};

pub type Request = tonic::Request<StartHandlerRequest>;
pub type Response = tonic::Response<HandlerStatesMapResponse>;

pub struct StarthandlerEndpoint<'a> {
    request: Request,
    mapping: RwLockWriteGuard<'a, Mapping>,
    server: &'a Server,
}

impl ServiceEndpoint<Request, Response> for StarthandlerEndpoint<'_> {
    fn execute(&self) -> Result<Response, tonic::Status> {
        let request = self.request.into_inner();
        let directory_path = request.directory_path.as_str();
        let mut states_map: HashMap<String, HandlerStateResponse> = HashMap::new();

        match self
            .mapping
            .clone()
            .directory_mapping
            .get_mut(directory_path)
        {
            Some(handler_mapping) => {
                if !handler_mapping.is_alive()
                    && self
                        .server
                        .is_concurrent_handlers_limit_reached(&self.mapping)
                {
                    return Err(tonic::Status::failed_precondition(format!(
                        "Aborted start handler - Reached concurrent live handler limit ({})",
                        self.server.config.concurrent_threads_limit
                    )));
                }
                let trace_tx = self.server.handlers_trace_tx.clone();
                let response =
                    self.mapping
                        .start_handler(directory_path, handler_mapping, trace_tx);
                states_map.insert(directory_path.to_owned(), response);
                Ok(Response::new(HandlerStatesMapResponse { states_map }))
            }
            None => {
                // If empty - All directories are requested
                if request.directory_path.is_empty() {
                    if self.mapping.directory_mapping.len()
                        > self.server.config.concurrent_threads_limit.into()
                    {
                        return Err(tonic::Status::failed_precondition(
                            format!("Aborted start handlers - Would pass concurrent live handler limit ({})\nCurrently live: {}", 
                            self.server.config.concurrent_threads_limit, self.mapping.get_live_handlers().count())));
                    }
                    for (directory_path, handler_mapping) in
                        self.mapping.clone().directory_mapping.iter_mut()
                    {
                        let trace_tx = self.server.handlers_trace_tx.clone();
                        let response =
                            self.mapping
                                .start_handler(directory_path, handler_mapping, trace_tx);
                        states_map.insert(directory_path.to_owned(), response);
                    }
                } else {
                    states_map.insert(
                        directory_path.to_owned(),
                        HandlerStateResponse {
                            is_alive: false,
                            message: String::from("Directory unhandled"),
                        },
                    );
                }
                Ok(Response::new(HandlerStatesMapResponse { states_map }))
            }
        }
    }
}
