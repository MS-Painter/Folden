use std::{collections::HashMap, convert::TryFrom};

use serde::{Serialize, Deserialize};
use tokio::sync::mpsc::Sender;

use generated_types::{HandlerChannelMessage, HandlerStatus};

// Mapping data used to handle known directories to handle
// If a handler thread has ceased isn't known at realtime rather will be verified via channel whenever needed to check given a client request

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mapping {
    pub directory_mapping: HashMap<String, HandlerMapping> // Hash map key binds to directory path
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