use std::{convert::TryFrom, fs, io, path::{Path, PathBuf}};

use clap::Values;
use serde::{Serialize, Deserialize};

use crate::{actions::WorkflowActions, event::WorkflowEvent};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub watch_recursive: bool,
    pub apply_on_startup_on_existing_files: bool,
    pub panic_handler_on_error: bool,
    pub event: WorkflowEvent,
    pub actions: Vec<WorkflowActions>
}

impl WorkflowConfig {
    pub fn default_new(events: Option<Values>, actions: Option<Values>) -> Self {
        Self {
            watch_recursive: false,
            apply_on_startup_on_existing_files: false,
            panic_handler_on_error: false,
            event: match events {
                Some(events) => WorkflowEvent::from(events),
                None => WorkflowEvent::default()
            },
            actions: match actions {
                Some(actions) => WorkflowActions::defaults(actions),
                None => vec![WorkflowActions::default()]
            }
        }
    }

    pub fn generate_config(&self, path: &Path) -> io::Result<()> {
        fs::write(path, toml::to_vec(*Box::new(self)).unwrap())
    }
    
    pub fn from_config(path: &PathBuf) -> Result<WorkflowConfig, toml::de::Error> {
        let data = fs::read(path).unwrap();
        Self::try_from(data)
    }
}

impl TryFrom<Vec<u8>> for WorkflowConfig {
    type Error = toml::de::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        toml::from_slice(&bytes)
    }
}