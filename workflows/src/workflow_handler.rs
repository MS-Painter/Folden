use std::fs;
use std::path::PathBuf;

use crossbeam::channel::Receiver;

use crate::actions::WorkflowAction;
use crate::workflow_config::WorkflowConfig;
use crate::workflow_execution_context::WorkflowExecutionContext;

pub struct WorkflowHandler {
    pub config: WorkflowConfig
}

impl WorkflowHandler {
    fn handle(&self, file_path: &PathBuf) {
        let mut context = WorkflowExecutionContext::new(file_path);
        for action in &self.config.actions {
            action.run(&mut context);
        }
    }

    fn on_startup(&self, path: &PathBuf) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            self.handle(&entry.path());
        }
        println!("Ended startup phase");
    }

    fn on_watch(&self, watcher_rx: Receiver<Result<notify::Event, notify::Error>>) {
        for result in watcher_rx {
            match result {
                Ok(event) => {
                    if self.config.event.is_handled_event(&event.kind) {
                        println!("Event to handle - {:?}", &event.kind);
                        let event_file_path = event.paths.first().unwrap();
                        self.handle(event_file_path);
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
        if self.config.apply_on_startup {
            self.on_startup(path);
        }
        self.on_watch(watcher_rx);
        println!("Ending watch");
    }
}