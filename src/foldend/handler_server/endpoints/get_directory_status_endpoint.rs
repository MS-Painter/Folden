use std::collections::HashMap;

use tokio::sync::RwLockReadGuard;

use super::handler_service_endpoint::ServiceEndpoint;
use crate::mapping::Mapping;
use generated_types::{GetDirectoryStatusRequest, HandlerSummary, HandlerSummaryMapResponse};

pub type Request = tonic::Request<GetDirectoryStatusRequest>;
pub type Response = tonic::Response<HandlerSummaryMapResponse>;

pub struct GetDirectoryStatusEndpoint<'a> {
    pub request: Request,
    pub mapping: RwLockReadGuard<'a, Mapping>,
}

impl<'a> GetDirectoryStatusEndpoint<'a> {
    pub fn new(request: Request, mapping: RwLockReadGuard<'a, Mapping>) -> Self {
        Self { request, mapping }
    }

    pub fn any_handler_alive(self) -> bool {
        if let Ok(response) = self.execute() {
            let response = response.into_inner();
            return response
                .summary_map
                .iter()
                .any(|(_dir, handler)| handler.is_alive);
        }
        false
    }
}

impl ServiceEndpoint<Request, Response> for GetDirectoryStatusEndpoint<'_> {
    fn execute(self) -> Result<Response, tonic::Status> {
        let request = self.request.get_ref();
        let directory_path = request.directory_path.as_str();
        let mut summary_map: HashMap<String, HandlerSummary> = HashMap::new();

        match self.mapping.directory_mapping.get(directory_path) {
            Some(handler_mapping) => {
                let state = handler_mapping.summary();
                summary_map.insert(directory_path.to_string(), state);
                Ok(Response::new(HandlerSummaryMapResponse { summary_map }))
            }
            None => {
                // If empty - All directories are requested
                if directory_path.is_empty() {
                    for (directory_path, handler_mapping) in self.mapping.directory_mapping.iter() {
                        summary_map.insert(directory_path.to_owned(), handler_mapping.summary());
                    }
                }
                Ok(Response::new(HandlerSummaryMapResponse { summary_map }))
            }
        }
    }
}
