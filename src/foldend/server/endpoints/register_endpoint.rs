use tonic::{Request, Response};

use super::handler_service_endpoint::ServiceEndpoint;
use generated_types::{HandlerStateResponse, RegisterToDirectoryRequest};

struct RegisterEndpoint;

impl ServiceEndpoint<Request<RegisterToDirectoryRequest>, Response<HandlerStateResponse>> for RegisterEndpoint {
    fn execute(&self) -> Result<Response<HandlerStateResponse>, tonic::Status> {
        todo!()
    }
}
