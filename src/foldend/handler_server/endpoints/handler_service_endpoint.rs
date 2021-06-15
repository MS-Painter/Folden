pub trait ServiceEndpoint<Request, Response> {
    fn execute(self) -> Result<Response, tonic::Status>;
}
