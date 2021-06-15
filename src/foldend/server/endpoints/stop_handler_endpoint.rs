use std::collections::HashMap;

use tokio::sync::RwLockWriteGuard;

use super::handler_service_endpoint::ServiceEndpoint;
use crate::{mapping::Mapping, server::Server};
use generated_types::{HandlerStateResponse, HandlerStatesMapResponse, StopHandlerRequest};

pub type Request = tonic::Request<StopHandlerRequest>;
pub type Response = tonic::Response<HandlerStatesMapResponse>;

pub struct StophandlerEndpoint<'a> {
    request: Request,
    mapping: RwLockWriteGuard<'a, Mapping>,
    server: &'a Server,
}

impl<'a> StophandlerEndpoint<'a> {
    pub fn new(
        request: Request,
        mapping: RwLockWriteGuard<'a, Mapping>,
        server: &'a Server,
    ) -> Self {
        Self {
            request,
            mapping,
            server,
        }
    }
}

impl ServiceEndpoint<Request, Response> for StophandlerEndpoint<'_> {
    fn execute(&self) -> Result<Response, tonic::Status> {
        let request = self.request.into_inner();
        let directory_path = request.directory_path.as_str();
        let mut states_map: HashMap<String, HandlerStateResponse> = HashMap::new();

        match self
            .mapping
            .clone()
            .directory_mapping
            .get_mut(&request.directory_path)
        {
            Some(handler_mapping) => {
                let response = self
                    .mapping
                    .stop_handler(
                        &self.server.config,
                        directory_path,
                        handler_mapping,
                        request.remove,
                    )
                    .await;
                states_map.insert(directory_path.to_owned(), response);
                Ok(Response::new(HandlerStatesMapResponse { states_map }))
            }
            None => {
                if request.directory_path.is_empty() {
                    // If empty - All directories are requested
                    for (directory_path, handler_mapping) in
                        self.mapping.clone().directory_mapping.iter_mut()
                    {
                        let response = self
                            .mapping
                            .stop_handler(
                                &self.server.config,
                                directory_path,
                                handler_mapping,
                                request.remove,
                            )
                            .await;
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
