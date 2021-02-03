use crate::Handler;

use tokio::sync::mpsc;
use mpsc::error::TryRecvError;
use serde::{Serialize, Deserialize};

use generated_types::HandlerChannelMessage;

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
    fn watch(&self, mut shutdown_channel_rx: mpsc::Receiver<HandlerChannelMessage> ) {
        let mut is_shutdown_required = false;
        while !is_shutdown_required {
            match shutdown_channel_rx.try_recv() {
                Ok(_val) => {
                    match _val  {
                        HandlerChannelMessage::Terminate => {
                            is_shutdown_required = true;
                        }
                        HandlerChannelMessage::Ping => {}
                    }
                    
                }
                Err(err) => {
                    match err {
                        TryRecvError::Empty => {
                            // TODO: Handler logic... 
                        }
                        TryRecvError::Closed => {
                            is_shutdown_required = true;
                        }
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