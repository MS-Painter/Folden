use crate::Handler;

use chrono::{DateTime, Utc};
use crossbeam::channel::Receiver;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveJoinHandler {
    max_parts: u8, // Max split archive parts supported to attempt rejoining
    max_file_size: u64, // Max file size to attempt computing
    naming_regex_match: String, // Only files matching regex will attempt computing
    #[serde(with = "custom_datetime_format")]
    from_date_created: DateTime<Utc>, // Compute only files created after given date (Date requires format as supplied from general project config)
}

impl Default for ArchiveJoinHandler {
    fn default() -> Self {
        Self { 
            max_parts: 2, 
            max_file_size: 50000, 
            naming_regex_match: String::from("*"), 
            from_date_created: Utc::now()
        }
    }
}

#[typetag::serde]
impl Handler for ArchiveJoinHandler {
    fn watch(&self, watcher_rx: Receiver<Result<notify::Event, notify::Error>>) {
        for result in watcher_rx {
            match result {
                Ok(event) => {
                    println!("eve - {:?}", event);
                    match event.kind {
                        notify::EventKind::Create(_) => {}
                        notify::EventKind::Remove(_) => {}
                        _ => {}
                    }
                }
                Err(error) => {
                    println!("error - {:?}", error);
                    match error.kind {
                        notify::ErrorKind::WatchNotFound => break,
                        _ => {}
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

mod custom_datetime_format {
    use chrono::{DateTime, Utc, TimeZone};
    use serde::{self, Deserialize, Serializer, Deserializer};
    
    const FORMAT: &'static str = "%d-%m-%Y %H:%M:%S";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S,) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D,) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}