use crate::Handler;

use crossbeam::channel::Receiver;
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ArchiveJoinHandler {
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

#[typetag::serde]
impl Handler for ArchiveJoinHandler {
    fn watch(&self, watcher_rx: Receiver<Result<notify::Event, notify::Error>>) {
        for result in watcher_rx {
            match result {
                Ok(event) => {
                    match event.kind {
                        notify::EventKind::Any => {}
                        notify::EventKind::Access(_) => {}
                        notify::EventKind::Create(_) => {}
                        notify::EventKind::Modify(_) => {}
                        notify::EventKind::Remove(_) => {}
                        notify::EventKind::Other => {}
                    }
                }
                Err(error) => {
                    match error.kind {
                        notify::ErrorKind::Generic(_) => {}
                        notify::ErrorKind::Io(_) => {}
                        notify::ErrorKind::PathNotFound => {}
                        notify::ErrorKind::WatchNotFound => {
                            break;
                        }
                        notify::ErrorKind::InvalidConfig(_) => {}
                    }
                }
            }
        }
        println!("Ending watch");
    }
}

impl From<Vec<u8>> for ArchiveJoinHandler {
    fn from(bytes: Vec<u8>) -> Self {
        toml::from_slice(&bytes).unwrap()
    }
}