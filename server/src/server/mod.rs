use std::sync::Arc;

use tokio::sync::{RwLock, mpsc};

use crate::mapping::Mapping;
use folder_handler::handlers_json::HandlersJson;
use crate::{config::Config, mapping::HandlerMapping};
use generated_types::{HandlerChannelMessage, HandlerStatus, HandlerSummary};

pub mod inter_process;

pub struct Server {
    pub config: Arc<Config>,
    pub mapping: Arc<RwLock<Mapping>>,
    pub handlers_json: Arc<HandlersJson>,
}

pub fn get_handler_summary(handler_mapping: &HandlerMapping) -> HandlerSummary {
    let mut state = HandlerSummary {
        state: HandlerStatus::Live as i32,
        type_name: handler_mapping.handler_type_name.clone(),
        config_path: handler_mapping.handler_config_path.clone(),
    };
    match handler_mapping.handler_thread_tx.clone() {
        Some(mut handler_thread_tx) => {
            match handler_thread_tx.try_send(HandlerChannelMessage::Ping) {
                Ok(_) => {}
                Err(err) => {
                    match err {
                        mpsc::error::TrySendError::Full(_) => {},
                        mpsc::error::TrySendError::Closed(_) => state.state = HandlerStatus::Dead as i32
                    }
                }
            }
        }
        None => state.state = HandlerStatus::Dead as i32
    }
    state
}