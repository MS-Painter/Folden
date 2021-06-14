use std::pin::Pin;
use std::{ops::Deref, sync::Arc};

use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tokio_stream::Stream;
use tonic::Request;

use crate::config::Config;
use crate::mapping::Mapping;
use generated_types::handler_service_server::HandlerService;

pub mod handler_service;

#[derive(Debug)]
pub struct Server {
    pub config: Arc<Config>,
    pub mapping: Arc<RwLock<Mapping>>,
    pub handlers_trace_tx:
        Arc<broadcast::Sender<Result<generated_types::TraceHandlerResponse, tonic::Status>>>,
}

impl Server {
    pub fn is_concurrent_handlers_limit_reached<T>(&self, mapping: &T) -> bool
    where
        T: Deref<Target = Mapping>,
    {
        let mut live_handlers_count: u8 = 0;
        if live_handlers_count >= self.config.concurrent_threads_limit {
            return true;
        }
        for handler_mapping in mapping.directory_mapping.values() {
            if handler_mapping.is_alive() {
                live_handlers_count += 1;
                if live_handlers_count >= self.config.concurrent_threads_limit {
                    return true;
                }
            }
        }
        false
    }

    fn convert_trace_channel_reciever_to_stream(&self) -> TraceHandlerStream {
        let mut rx = self.handlers_trace_tx.subscribe();
        Box::pin(async_stream::stream! {
            while let Ok(item) = rx.recv().await {
                yield item;
            }
        })
    }

    async fn is_any_handler_alive(&self) -> bool {
        let response =
            self.get_directory_status(Request::new(generated_types::GetDirectoryStatusRequest {
                directory_path: String::new(),
            }));
        if let Ok(response) = response.await {
            let response = response.into_inner();
            return response
                .summary_map
                .iter()
                .any(|(_dir, handler)| handler.is_alive);
        }
        false
    }
}

pub type TraceHandlerStream = Pin<
    Box<
        dyn Stream<Item = Result<generated_types::TraceHandlerResponse, tonic::Status>>
            + Send
            + Sync
            + 'static,
    >,
>;
