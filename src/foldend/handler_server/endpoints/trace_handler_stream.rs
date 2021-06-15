use std::pin::Pin;

use tokio_stream::Stream;

pub type TraceHandlerStream = Pin<
    Box<
        dyn Stream<Item = Result<generated_types::TraceHandlerResponse, tonic::Status>>
            + Send
            + Sync
            + 'static,
    >,
>;
