use std::collections::HashMap;

use serde::{Serialize, Deserialize};

// Mapping data used to handle known directories to handle

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Mapping {
    pub directory_mapping: HashMap<String, HandlerMapping> // Hash map key binds to directory path
}

impl From<Vec<u8>> for Mapping {
    fn from(bytes: Vec<u8>) -> Self {
        serde_json::from_slice(&bytes).unwrap()
    }
}

impl Into<Vec<u8>> for Mapping {
    fn into(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HandlerMapping {
    #[serde(default)]
    pub handler_thread_id: u32,
    #[serde(default)]
    pub handler_type: String,
    #[serde(default)]
    pub handler_config_path: String,
}