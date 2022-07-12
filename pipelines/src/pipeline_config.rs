use std::{convert::TryFrom, fs, io, path::Path};

use clap::Values;
use serde::{Deserialize, Serialize};

use crate::{actions::PipelineActions, event::PipelineEvent};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub watch_recursive: bool,
    pub apply_on_startup_on_existing_files: bool,
    pub stop_handler_on_error: bool,
    pub event: PipelineEvent,
    pub actions: Vec<PipelineActions>,
}

impl PipelineConfig {
    pub fn default_new(events: Option<Values>, actions: Option<Values>) -> Self {
        Self {
            watch_recursive: false,
            apply_on_startup_on_existing_files: false,
            stop_handler_on_error: false,
            event: match events {
                Some(events) => PipelineEvent::from(events),
                None => PipelineEvent::default(),
            },
            actions: match actions {
                Some(actions) => PipelineActions::defaults(actions),
                None => vec![PipelineActions::default()],
            },
        }
    }

    pub fn generate_config(&self, path: &Path) -> io::Result<()> {
        fs::write(path, toml::to_vec(*Box::new(self)).unwrap())
    }
}

impl TryFrom<Vec<u8>> for PipelineConfig {
    type Error = toml::de::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        toml::from_slice(&bytes)
    }
}
