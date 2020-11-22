use crate::Handler;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
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

fn datetime_default() -> String {
    "%d/%m/%yyyy".to_string()
}

impl Handler for ArchiveJoinHandler{
    fn new() -> Self {
        ArchiveJoinHandler{
            max_parts: 0,
            max_file_size: 0,
            naming_regex_match: "".to_string(),
            from_date_created: "".to_string()
        }
    }

    fn from_bytes(bytes: Vec<u8>) -> Self {
        toml::from_slice(&bytes).unwrap()
    }

    fn watch(&self) {
        unimplemented!()
    }
}
