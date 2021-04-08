use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Local};
use crossbeam::channel::Receiver;

use crate::workflow_config::WorkflowConfig;

pub struct WorkflowHandler {
    pub config: WorkflowConfig
}

impl WorkflowHandler {
    fn on_startup(&self, path: &PathBuf) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let file_creation_time: DateTime<Local> = DateTime::from(entry.metadata().unwrap().created().unwrap());
            if self.config.event.from_date_created <= file_creation_time {
                println!("{:?}", entry.file_name());
            }
        }
        println!("Ended startup phase");
    }

    fn on_watch(&self, watcher_rx: Receiver<Result<notify::Event, notify::Error>>) {
        for result in watcher_rx {
            match result {
                Ok(event) => {
                    if self.config.event.is_handled_event(&event.kind) {
                        println!("Event to handle!");
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

    pub fn watch(&mut self, path: &PathBuf, watcher_rx: Receiver<Result<notify::Event, notify::Error>>) {
        self.on_startup(path);
        self.on_watch(watcher_rx);
        println!("Ending watch");
    }
}