use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use tokio::sync::mpsc::Sender;

use generated_types::HandlerChannelMessage;

// Mapping data used to handle known directories to handle
// If a handler thread has ceased isn't known at realtime rather will be verified via channel whenever needed to check given a client request

// TODO: Find a way to de/serialize a reciever to allow storing for later usage. Or omit such data to serialize to recreate reciever anyway!

#[derive(Debug)]
pub struct Mapping {
    pub directory_mapping: HashMap<String, HandlerMapping> // Hash map key binds to directory path
}

#[derive(Debug)]
pub struct HandlerMapping {
    pub handler_thread_tx: Sender<HandlerChannelMessage>, // Channel sender providing thread health and allowing manual thread shutdown
    pub handler_type_name: String,
    pub handler_config_path: String,
}