use tokio::sync::RwLockWriteGuard;

use super::handler_service_endpoint::ServiceEndpoint;
use crate::{mapping::Mapping, server::Server};
use generated_types::ModifyHandlerRequest;

pub type Request = tonic::Request<ModifyHandlerRequest>;
pub type Response = tonic::Response<()>;

pub struct ModifyEndpoint<'a> {
    request: Request,
    mapping: RwLockWriteGuard<'a, Mapping>,
    server: &'a Server,
}

impl<'a> ModifyEndpoint<'a> {
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

impl ServiceEndpoint<Request, Response> for ModifyEndpoint<'_> {
    fn execute(mut self) -> Result<Response, tonic::Status> {
        let request = self.request.get_ref();
        match self
            .mapping
            .directory_mapping
            .get_mut(&request.directory_path)
        {
            Some(handler_mapping) => handler_mapping.modify(&request),
            None => {
                if request.directory_path.is_empty() {
                    // If empty - All directories are requested
                    for handler_mapping in self.mapping.directory_mapping.values_mut() {
                        handler_mapping.modify(&request);
                    }
                } else {
                    return Err(tonic::Status::not_found(
                        "Directory isn't registered to handle",
                    ));
                }
            }
        }

        match self.mapping.save(&self.server.config.mapping_state_path) {
            Ok(result) => Ok(Response::new(result)),
            Err(e) => Err(tonic::Status::unknown(format!(
                "Failed to save modifications to mapping file.\nErr - {:?}",
                e
            ))),
        }
    }
}
