use std::{collections::HashMap, convert::TryFrom};

use serde::{Serialize, Deserialize};
use tokio::sync::mpsc::Sender;

use generated_types::{HandlerChannelMessage, handler_summary};

// Mapping data used to handle known directories to handle
// If a handler thread has ceased isn't known at realtime rather will be verified via channel whenever needed to check given a client request

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct HandlerMapping {
    #[serde(skip)]
    pub handler_thread_tx: Option<Sender<HandlerChannelMessage>>, // Channel sender providing thread health and allowing manual thread shutdown
    pub handler_type_name: String,
    pub handler_config_path: String,
}

impl HandlerMapping {
    pub fn status(&self) -> handler_summary::Status {
        match self.handler_thread_tx.clone() {
            Some(mut handler_thread_tx) => {
                match handler_thread_tx.try_send(HandlerChannelMessage::Ping) {
                    Ok(_) => handler_summary::Status::Live,
                    Err(error) => {
                        match error {
                            tokio::sync::mpsc::error::TrySendError::Full(_) => handler_summary::Status::Live,
                            tokio::sync::mpsc::error::TrySendError::Closed(_) => handler_summary::Status::Dead,
                        }
                    }
                }
            }
            None => handler_summary::Status::Dead
        }
    }
}