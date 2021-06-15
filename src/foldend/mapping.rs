use std::{collections::HashMap, convert::TryFrom, fs, path::Path, result::Result, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::{config::Config, handler_mapping::HandlerMapping};
use generated_types::HandlerStateResponse;

// Mapping data used to handle known directories to handle
// If a handler thread has ceased isn't known at realtime rather will be verified via channel whenever needed to check given a client request

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mapping {
    pub directory_mapping: HashMap<String, HandlerMapping>, // Hash map key binds to directory path
}

impl Mapping {
    pub fn save(&self, mapping_state_path: &Path) -> Result<(), std::io::Error> {
        let mapping_data: Vec<u8> = self.into();
        fs::write(mapping_state_path, mapping_data)
    }

    pub fn iter_live_handlers(&self) -> impl Iterator<Item = (&String, &HandlerMapping)> {
        self.directory_mapping
            .iter()
            .filter(|(_dir, mapping)| mapping.is_alive())
    }

    pub fn start_handler(
        &mut self,
        directory_path: &str,
        handler_mapping: &mut HandlerMapping,
        trace_tx: Arc<
            broadcast::Sender<Result<generated_types::TraceHandlerResponse, tonic::Status>>,
        >,
    ) -> HandlerStateResponse {
        if handler_mapping.is_alive() {
            HandlerStateResponse {
                is_alive: true,
                message: String::from("Handler already up"),
            }
        } else {
            match handler_mapping.spawn_handler_thread(directory_path.to_string(), trace_tx) {
                Ok(_) => {
                    self.directory_mapping
                        .insert(directory_path.to_string(), handler_mapping.to_owned());
                    HandlerStateResponse {
                        is_alive: true,
                        message: String::from("Started handler"),
                    }
                }
                Err(err) => HandlerStateResponse {
                    is_alive: false,
                    message: format!("Failed to start handler.\nError: {}", err),
                },
            }
        }
    }

    pub fn stop_handler(
        &mut self,
        config: &Config,
        directory_path: &str,
        handler_mapping: &mut HandlerMapping,
        remove: bool,
    ) -> HandlerStateResponse {
        if handler_mapping.is_alive() {
            match handler_mapping.stop_handler_thread() {
                Ok(mut message) => {
                    if remove {
                        self.directory_mapping.remove(directory_path);
                        message.push_str(" & removed");
                        let _result = self.save(&config.mapping_state_path);
                    } else {
                        handler_mapping.watcher_tx = None;
                    }
                    HandlerStateResponse {
                        is_alive: false,
                        message,
                    }
                }
                Err(message) => HandlerStateResponse {
                    is_alive: true,
                    message,
                },
            }
        } else {
            let mut message = String::from("Handler already stopped");
            if remove {
                self.directory_mapping.remove(directory_path);
                message.push_str(" & removed");
                let _result = self.save(&config.mapping_state_path);
            } else {
                handler_mapping.watcher_tx = None;
            }
            HandlerStateResponse {
                is_alive: false,
                message,
            }
        }
    }
}

impl TryFrom<Vec<u8>> for Mapping {
    type Error = &'static str;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        match toml::from_slice(&bytes) {
            Ok(mapping) => Ok(mapping),
            Err(_) => Err("Couldn't deserialize data to mapping"),
        }
    }
}

impl From<Mapping> for Vec<u8> {
    fn from(val: Mapping) -> Self {
        toml::to_vec(&val).unwrap()
    }
}

impl From<&Mapping> for Vec<u8> {
    fn from(val: &Mapping) -> Self {
        toml::to_vec(val).unwrap()
    }
}
