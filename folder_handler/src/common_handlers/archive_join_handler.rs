use std::{fs, path::PathBuf};

use chrono::{DateTime, Local};
use crossbeam::channel::Receiver;
use serde::{Serialize, Deserialize};

use crate::Handler;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchiveJoinHandler {
    max_parts: u8, // Max split archive parts supported to attempt rejoining
    max_file_size: u64, // Max file size to attempt computing
    naming_regex_match: String, // Only files matching regex will attempt computing
    #[serde(with = "custom_datetime_format")]
    from_date_created: DateTime<Local>, // Compute only files created after given date (Date requires format as supplied from general project config)
}

impl ArchiveJoinHandler {
    fn on_startup(&self, path: &PathBuf) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let file_creation_time: DateTime<Local> = DateTime::from(entry.metadata().unwrap().created().unwrap());
            if self.from_date_created <= file_creation_time {
                println!("{:?}", entry.file_name());
            }
        }
        println!("Ended startup phase");
    }

    fn on_watch(&self, watcher_rx: Receiver<Result<notify::Event, notify::Error>>) {
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
    }
}

impl Default for ArchiveJoinHandler {
    fn default() -> Self {
        Self { 
            max_parts: 2, 
            max_file_size: 50000, 
            naming_regex_match: String::from("*"), 
            from_date_created: Local::now()
        }
    }
}

#[typetag::serde]
impl Handler for ArchiveJoinHandler {
    fn watch(&mut self, path: &PathBuf, config_path: &PathBuf, watcher_rx: Receiver<Result<notify::Event, notify::Error>>) {
        self.from_config(config_path);
        self.on_startup(path);
        self.on_watch(watcher_rx);
        println!("Ending watch");
    }
}

impl From<Vec<u8>> for ArchiveJoinHandler {
    fn from(bytes: Vec<u8>) -> Self {
        toml::from_slice(&bytes).unwrap()
    }
}

mod custom_datetime_format {
    use chrono::{DateTime, Local, TimeZone};
    use serde::{self, Deserialize, Serializer, Deserializer};
    
    const FORMAT: &'static str = "%d-%m-%Y %H:%M:%S";

    pub fn serialize<S>(date: &DateTime<Local>, serializer: S,) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D,) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Local.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}