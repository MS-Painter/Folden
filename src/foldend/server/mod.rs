use std::{ops::Deref, sync::Arc};

use tokio::sync::{mpsc, RwLock};

use crate::config::Config;
use crate::mapping::Mapping;

pub mod handler_service;

#[derive(Debug)]
pub struct Server {
    pub config: Arc<Config>,
    pub mapping: Arc<RwLock<Mapping>>,
    pub handlers_trace_tx: Arc<Option<mpsc::Sender<Result<generated_types::TraceHandlerResponse, tonic::Status>>>>,
    pub handlers_trace_rx: Arc<Option<mpsc::Receiver<Result<generated_types::TraceHandlerResponse, tonic::Status>>>>,
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
}