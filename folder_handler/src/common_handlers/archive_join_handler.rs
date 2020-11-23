use crate::{Handler};

use serde::{Serialize, Deserialize};
use std::{fs, io};
use std::path::Path;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ArchiveJoinHandler{
    #[serde(default)]
    max_parts: u8, // Max split archive parts supported to attempt rejoining
    #[serde(default)]
    max_file_size: u64, // Max file size to attempt computing
    #[serde(default)]
    naming_regex_match: String, // Only files matching regex will attempt computing
    #[serde(default = "datetime_default")]
    from_date_created: String, // Compute only files created after given date (Date requires format as supplied from general project config)
}

impl ArchiveJoinHandler{
    pub(crate) fn generate_config(&self, path: &Path) -> io::Result<()>{
        fs::write(path, toml::to_vec(&self).unwrap())
    }
}

fn datetime_default() -> String {
    "%d/%m/%yyyy".to_string()
}

#[typetag::serde]
impl Handler for ArchiveJoinHandler{
    fn watch(&self) {
        unimplemented!()
    }
}

impl From<Vec<u8>> for ArchiveJoinHandler {
    fn from(bytes: Vec<u8>) -> Self {
        toml::from_slice(&bytes).unwrap()
    }
}