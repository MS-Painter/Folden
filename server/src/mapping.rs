use std::{collections::HashMap, convert::TryFrom, sync::Arc, thread};

use serde::{Serialize, Deserialize};
use tokio::sync::mpsc::{self, Sender};

use folder_handler::handlers_json::HandlersJson;
use generated_types::{HandlerChannelMessage, HandlerStateResponse, HandlerStatus};

// Mapping data used to handle known directories to handle
// If a handler thread has ceased isn't known at realtime rather will be verified via channel whenever needed to check given a client request

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mapping {
    pub directory_mapping: HashMap<String, HandlerMapping> // Hash map key binds to directory path
}

impl Mapping {
    pub fn spawn_handler(&mut self, handlers_json: Arc<HandlersJson>, directory_path: &str, handler_mapping: &HandlerMapping) -> HandlerStateResponse {
        match handler_mapping.status() {
            HandlerStatus::Dead => {
                let handler_type_name = handler_mapping.handler_type_name.clone();
                let handler_config_path = handler_mapping.handler_config_path.clone();
                self.spawn_handler_thread(handlers_json, directory_path.to_string(), handler_type_name, handler_config_path);
                HandlerStateResponse {
                    state: HandlerStatus::Live as i32,
                    message: String::from("Handler started"),
                }
            }
            HandlerStatus::Live => HandlerStateResponse {
                state: HandlerStatus::Live as i32,
                message: String::from("Handler already up"),
            },
        }
    }
    
    pub fn spawn_handler_thread(&mut self, handlers_json: Arc<HandlersJson>, directory_path: String, handler_type_name: String, handler_config_path: String) {
        match handlers_json.get_handler_by_name(&handler_type_name) {
            Ok(handler) => {
                let (tx, rx) = mpsc::channel::<HandlerChannelMessage>(2);
                thread::spawn(move || {
                    let handler = handler;
                    let rx = rx;
                    handler.watch(rx);
                });            
                // Insert or update the value of the current handled directory
                self.directory_mapping.insert(directory_path, HandlerMapping {
                    handler_thread_tx: Option::Some(tx),
                    handler_type_name,
                    handler_config_path,
                });
            },
            Err(e) => panic!(e)
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

impl Into<Vec<u8>> for Mapping {
    fn into(self) -> Vec<u8> {
        toml::to_vec(&self).unwrap()
    }
}

impl Into<Vec<u8>> for &Mapping {
    fn into(self) -> Vec<u8> {
        toml::to_vec(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HandlerMapping {
    #[serde(skip)]
    pub handler_thread_tx: Option<Sender<HandlerChannelMessage>>, // Channel sender providing thread health and allowing manual thread shutdown
    pub handler_type_name: String,
    pub handler_config_path: String,
}

impl HandlerMapping {
    pub fn status(&self) -> HandlerStatus {
        match self.handler_thread_tx.clone() {
            Some(mut handler_thread_tx) => {
                match handler_thread_tx.try_send(HandlerChannelMessage::Ping) {
                    Ok(_) => HandlerStatus::Live,
                    Err(error) => {
                        match error {
                            tokio::sync::mpsc::error::TrySendError::Full(_) => HandlerStatus::Live,
                            tokio::sync::mpsc::error::TrySendError::Closed(_) => HandlerStatus::Dead,
                        }
                    }
                }
            }
            None => HandlerStatus::Dead
        }
    }

    pub async fn stop_handler_thread(&self) -> Result<String, String> {
        match self.handler_thread_tx.clone().unwrap().send(HandlerChannelMessage::Terminate).await {
            Ok(_) => {
                Ok(String::from("Handler stopped"))
            }
            Err(error) => {
                let mut message = String::from("Failed to stop handler\nError: ");
                message.push_str(error.to_string().as_str());
                Err(message)
            }
        }
    }
}