use std::fs;
use std::path::PathBuf;

use crossbeam::channel::Receiver;

use crate::actions::WorkflowAction;
use crate::workflow_config::WorkflowConfig;

pub struct WorkflowHandler {
    pub config: WorkflowConfig
}

impl WorkflowHandler {
    fn on_startup(&self, path: &PathBuf) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if self.config.apply_on_startup {
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
                        for action in &self.config.actions {
                            println!("{:?}", action);
                            action.run();
                        }
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