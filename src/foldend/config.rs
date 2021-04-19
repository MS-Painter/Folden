use std::{convert::TryFrom, fs, path::PathBuf};

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub mapping_state_path: PathBuf,
}

impl Config {
    pub fn save(&self, file_path: &PathBuf) -> Result<(), std::io::Error> {
        let config_data: Vec<u8> = self.into();
        fs::write(file_path, config_data)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mapping_state_path: PathBuf::from("foldend_mapping.toml"),
        }
    }
}

impl TryFrom<Vec<u8>> for Config {
    type Error = toml::de::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        toml::from_slice(&bytes)
    }
}

impl Into<Vec<u8>> for &Config {
    fn into(self) -> Vec<u8> {
        toml::to_vec(&self).unwrap()
    }
}
