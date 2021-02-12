use std::{fs, io::ErrorKind, ops::Deref, sync::Arc, thread};

use tokio::sync::{RwLock, RwLockWriteGuard, mpsc};

use crate::mapping::Mapping;
use generated_types::{HandlerChannelMessage, HandlerSummary, HandlerStatus};
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

pub fn start_handler_thread(
    mut mapping: RwLockWriteGuard<Mapping>, handlers_json: Arc<HandlersJson>, 
    directory_path: String, handler_type_name: String, handler_config_path: String) {
    match handlers_json.get_handler_by_name(&handler_type_name) {
        Ok(handler) => {
            let (tx, rx) = mpsc::channel::<HandlerChannelMessage>(2);
            thread::spawn(move || {
                let handler = handler;
                let rx = rx;
                handler.watch(rx);
            });            
            // Insert or update the value of the current handled directory
            mapping.directory_mapping.insert(directory_path, HandlerMapping {
                handler_thread_tx: Option::Some(tx),
                handler_type_name,
                handler_config_path,
            });
        },
        Err(e) => panic!(e)
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