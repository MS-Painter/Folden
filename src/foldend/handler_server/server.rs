use std::sync::Arc;

use tokio::sync::{broadcast, RwLock, RwLockReadGuard};

use super::endpoints::trace_handler_stream;
use crate::config::Config;
use crate::handler_mapping::HandlerMapping;
use crate::mapping::Mapping;

#[derive(Debug)]
pub struct Server {
    pub config: Arc<Config>,
    pub mapping: Arc<RwLock<Mapping>>,
    pub handlers_trace_tx:
        Arc<broadcast::Sender<Result<generated_types::TraceHandlerResponse, tonic::Status>>>,
}

impl Server {
    pub fn convert_trace_channel_reciever_to_stream(
        &self,
    ) -> trace_handler_stream::TraceHandlerStream {
        let mut rx = self.handlers_trace_tx.subscribe();
        Box::pin(async_stream::stream! {
            while let Ok(item) = rx.recv().await {
                yield item;
            }
        })
    }

    pub fn get_handler<'a>(
        &self,
        mapping: &'a RwLockReadGuard<Mapping>,
        directory_path: &str,
        is_required_alive: bool,
    ) -> Result<&'a HandlerMapping, tonic::Status> {
        match mapping.directory_mapping.get(directory_path) {
            Some(handler_mapping) => {
                if is_required_alive && !&handler_mapping.is_alive() {
                    return Err(tonic::Status::failed_precondition("Handler isn't alive"));
                }
                Ok(handler_mapping)
            }
            None => Err(tonic::Status::not_found(
                "Directory isn't registered to handle",
            )),
        }
    }
}
