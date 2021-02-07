use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use tokio::sync::mpsc::Sender;

use generated_types::HandlerChannelMessage;

// Mapping data used to handle known directories to handle
// If a handler thread has ceased isn't known at realtime rather will be verified via channel whenever needed to check given a client request

#[derive(Debug, Serialize, Deserialize)]
pub struct Mapping {
    pub directory_mapping: HashMap<String, HandlerMapping> // Hash map key binds to directory path
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HandlerMapping {
    #[serde(skip)]
    pub handler_thread_tx: Option<Sender<HandlerChannelMessage>>, // Channel sender providing thread health and allowing manual thread shutdown
    pub handler_type_name: String,
    pub handler_config_path: String,
}