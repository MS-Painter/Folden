use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub mapping_state_path: PathBuf,
    pub mapping_status_strategy: MappingStatusStrategy,  
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mapping_state_path: PathBuf::from("foldend_mapping.toml"),
            mapping_status_strategy: MappingStatusStrategy::Continue,
        }
    }
}

impl From<Vec<u8>> for Config {
    fn from(bytes: Vec<u8>) -> Self {
        toml::from_slice(&bytes).unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub enum MappingStatusStrategy {
    None,
    Save, // Saves registered handlers to mapping file
    Continue // (Save strategy) + On startup start registered handlers
}
