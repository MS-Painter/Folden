use std::pin::Pin;
use std::{ops::Deref, sync::Arc};

use tokio::sync::RwLock;
use tokio_stream::Stream;
use tokio::sync::broadcast;

use crate::config::Config;
use crate::mapping::Mapping;

pub mod handler_service;

#[derive(Debug)]
pub struct Server {
    pub config: Arc<Config>,
    pub mapping: Arc<RwLock<Mapping>>,
    pub handlers_trace_tx: Arc<broadcast::Sender<Result<generated_types::TraceHandlerResponse, tonic::Status>>>,
}

impl Server {
    pub fn is_concurrent_handlers_limit_reached<T>(&self, mapping: &T) -> bool where T: Deref<Target = Mapping> {
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
        tracing::info!("{}", self.handlers_trace_tx.receiver_count());
        // Convert the channels to a `Stream`.
        Box::pin(async_stream::stream! {
            while let Ok(item) = rx.recv().await {
                yield item;
            }
        })
    }
}

pub type TraceHandlerStream = Pin<Box<dyn Stream<Item = Result<generated_types::TraceHandlerResponse, tonic::Status>> + Send + Sync + 'static>>;