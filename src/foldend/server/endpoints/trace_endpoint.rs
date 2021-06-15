use tokio::sync::RwLockReadGuard;

use super::{handler_service_endpoint::ServiceEndpoint, trace_handler_stream::TraceHandlerStream};
use crate::{
    mapping::Mapping,
    server::{endpoints::get_directory_status_endpoint::GetDirectoryStatusEndpoint, Server},
};
use generated_types::{GetDirectoryStatusRequest, TraceHandlerRequest};

pub type Request = tonic::Request<TraceHandlerRequest>;
pub type Response = tonic::Response<TraceHandlerStream>;

pub struct TraceEndpoint<'a> {
    request: Request,
    mapping: RwLockReadGuard<'a, Mapping>,
    server: &'a Server,
}

impl<'a> TraceEndpoint<'a> {
    pub fn new(
        request: Request,
        mapping: RwLockReadGuard<'a, Mapping>,
        server: &'a Server,
    ) -> Self {
        Self {
            request,
            mapping,
            server,
        }
    }
}

impl ServiceEndpoint<Request, Response> for TraceEndpoint<'_> {
    fn execute(&self) -> Result<Response, tonic::Status> {
        let request = self.request.into_inner();
        // If empty - All directories are requested
        if !request.directory_path.is_empty() {
            if let Err(e) = self
                .server
                .get_handler(&self.mapping, &request.directory_path, true)
            {
                return Err(e);
            }
        } else if self.mapping.directory_mapping.is_empty() {
            return Err(tonic::Status::not_found(
                "No handler registered to filesystem to trace",
            ));
        } else {
            let get_dir_status_endpoint = GetDirectoryStatusEndpoint {
                request: tonic::Request::new(GetDirectoryStatusRequest {
                    directory_path: String::new(),
                }),
                mapping: self.mapping,
            };
            if !get_dir_status_endpoint.is_any_handler_alive() {
                return Err(tonic::Status::not_found("No handler is alive to trace"));
            }
        }

        let rx_stream = self.server.convert_trace_channel_reciever_to_stream();
        tracing::debug!(
            "Handler trace receivers live: {}",
            self.server.handlers_trace_tx.receiver_count()
        );
        return Ok(Response::new(rx_stream));
    }
}
