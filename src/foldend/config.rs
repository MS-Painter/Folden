use std::{
    convert::TryFrom,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use folden::shared_config::DEFAULT_PORT;

pub const DEFAULT_CONCURRENT_THREADS_LIMIT: u8 = 10;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub mapping_state_path: PathBuf,
    pub tracing_file_path: PathBuf,
    pub concurrent_threads_limit: u8,
    #[serde(skip)]
    pub port: u16,
}

impl Config {
    pub fn save(&self, file_path: &Path) -> Result<(), std::io::Error> {
        let config_data: Vec<u8> = self.into();
        fs::write(file_path, config_data)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mapping_state_path: PathBuf::from("foldend_mapping.toml"),
            tracing_file_path: PathBuf::from("foldend.log"),
            concurrent_threads_limit: DEFAULT_CONCURRENT_THREADS_LIMIT,
            port: DEFAULT_PORT.parse().unwrap(),
        }
    }
}

impl TryFrom<Vec<u8>> for Config {
    type Error = toml::de::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        toml::from_slice(&bytes)
    }
}

impl From<&Config> for Vec<u8> {
    fn from(val: &Config) -> Self {
        toml::to_vec(&val).unwrap()
    }
}
