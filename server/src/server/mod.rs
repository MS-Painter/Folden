use std::{fs, io::ErrorKind, ops::Deref, sync::Arc, thread};

use tokio::sync::{RwLock, RwLockWriteGuard, mpsc};

use crate::mapping::Mapping;
use generated_types::{HandlerChannelMessage, HandlerStatus, HandlerSummary};
use folder_handler::handlers_json::HandlersJson;
use crate::{config::Config, mapping::HandlerMapping};

pub mod inter_process;

pub struct Server {
    pub config: Arc<Config>,
    pub mapping: Arc<RwLock<Mapping>>,
    pub handlers_json: Arc<HandlersJson>,
}

impl Server {
    pub async fn save_mapping(&self) -> Result<(), std::io::Error> {
        match self.config.mapping_status_strategy {
            crate::config::MappingStatusStrategy::None => Err(std::io::Error::new(ErrorKind::Other, "Not allowed current in config state")),
            _ => {
                let mapping = self.mapping.read().await;
                let mapping = mapping.deref();
                let mapping_data: Vec<u8> = mapping.into();
                fs::write(&self.config.mapping_state_path, mapping_data)
            }
        }
        
    }
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