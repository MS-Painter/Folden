use tokio::sync::RwLockWriteGuard;

use super::handler_service_endpoint::ServiceEndpoint;
use crate::{handler_mapping::HandlerMapping, mapping::Mapping, server::Server};
use generated_types::{HandlerStateResponse, RegisterToDirectoryRequest};

pub type Request = tonic::Request<RegisterToDirectoryRequest>;
pub type Response = tonic::Response<HandlerStateResponse>;

pub struct RegisterEndpoint<'a> {
    request: Request,
    mapping: RwLockWriteGuard<'a, Mapping>,
    server: &'a Server,
}

impl ServiceEndpoint<Request, Response> for RegisterEndpoint<'_> {
    fn execute(&self) -> Result<Response, tonic::Status> {
        let request = self.request.into_inner();
        let request_directory_path = request.directory_path.as_str();
        if self
            .mapping
            .directory_mapping
            .get(request_directory_path)
            .is_some()
        {
            Ok(Response::new(HandlerStateResponse {
                is_alive: true,
                message: String::from("Directory already handled by handler"),
            }))
        } else {
            // Check if requested directory is a child / parent of any handled directory
            for directory_path in self.mapping.directory_mapping.keys() {
                if request_directory_path.contains(directory_path) {
                    return Ok(Response::new(HandlerStateResponse {
                        is_alive: false,
                        message: format!(
                            "Couldn't register\nDirectory is a child of handled directory - {}",
                            directory_path
                        ),
                    }));
                } else if directory_path.contains(request_directory_path) {
                    return Ok(Response::new(HandlerStateResponse {
                        is_alive: false,
                        message: format!(
                            "Couldn't register\nDirectory is a parent of requested directory - {}",
                            directory_path
                        ),
                    }));
                }
            }
            let mut handler_mapping = HandlerMapping::new(
                request.handler_config_path,
                request.is_auto_startup,
                String::new(),
            );
            if request.is_start_on_register {
                if self
                    .server
                    .is_concurrent_handlers_limit_reached(&self.mapping)
                {
                    self.mapping
                        .directory_mapping
                        .insert(request.directory_path, handler_mapping);
                    let _result = self.mapping.save(&self.server.config.mapping_state_path);
                    return Ok(Response::new(HandlerStateResponse {
                        is_alive: false,
                        message: format!("Registered handler without starting - Reached concurrent live handler limit ({})", self.server.config.concurrent_threads_limit),
                    }));
                }
                let trace_tx = self.server.handlers_trace_tx.clone();
                match self.mapping.spawn_handler_thread(
                    request.directory_path,
                    &mut handler_mapping,
                    trace_tx,
                ) {
                    Ok(_) => {
                        let _result = self.mapping.save(&self.server.config.mapping_state_path);
                        Ok(Response::new(HandlerStateResponse {
                            is_alive: true,
                            message: String::from("Registered and started handler"),
                        }))
                    }
                    Err(err) => Ok(Response::new(HandlerStateResponse {
                        is_alive: false,
                        message: format!("Failed to register and start handler.\nError: {}", err),
                    })),
                }
            } else {
                self.mapping
                    .directory_mapping
                    .insert(request.directory_path, handler_mapping);
                let _result = self.mapping.save(&self.server.config.mapping_state_path);
                Ok(Response::new(HandlerStateResponse {
                    is_alive: false,
                    message: String::from("Registered handler"),
                }))
            }
        }
    }
}
