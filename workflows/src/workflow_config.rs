use serde::{Serialize, Deserialize};
use std::{fs, io, path::{Path, PathBuf}};

use crate::event::WorkflowEvent;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub event: WorkflowEvent,
}

impl WorkflowConfig {
    pub fn generate_config(&self, path: &Path) -> io::Result<()> {
        fs::write(path, toml::to_vec(*Box::new(self)).unwrap())
    }
    
    pub fn from_config(&mut self, path: &PathBuf) {
        let data = fs::read(path).unwrap();
        let config_clone = Self::from(data);
        self.clone_from(&config_clone);
    }
}

impl From<Vec<u8>> for WorkflowConfig {
    fn from(bytes: Vec<u8>) -> Self {
        toml::from_slice(&bytes).unwrap()
    }
}