use tonic::{Request, Response};

use super::Server;
use generated_types::{
    inter_process_server::InterProcess, 
    GetDirectoryStatusRequest, GetDirectoryStatusResponse, 
    RegisterToDirectoryRequest, RegisterToDirectoryResponse
};

#[tonic::async_trait]
impl InterProcess for Server {
    async fn register_to_directory(&self, request:Request<RegisterToDirectoryRequest>) ->
    Result<Response<RegisterToDirectoryResponse>,tonic::Status> {
        let request = request.into_inner();
        println!("{}", request.directory_path);
        println!("{}", request.handler_type_name);

        Ok(Response::new(RegisterToDirectoryResponse {
            message: "".to_string(),
        }))
    }

    async fn get_directory_status(&self, request:Request<GetDirectoryStatusRequest>) ->
    Result<Response<GetDirectoryStatusResponse>,tonic::Status> {
        let request = request.into_inner();
        println!("{}", request.directory_path);

        Ok(Response::new(GetDirectoryStatusResponse {
            message: "".to_string(),
        }))
    }
}