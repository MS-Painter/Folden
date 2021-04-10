use std::fs;
use std::path::PathBuf;

use regex::Regex;
use crossbeam::channel::Receiver;

use crate::actions::WorkflowAction;
use crate::workflow_config::WorkflowConfig;
use crate::workflow_execution_context::WorkflowExecutionContext;

pub struct WorkflowHandler {
    pub config: WorkflowConfig
}

impl WorkflowHandler {
    fn handle(&self, file_path: &PathBuf, re: &Regex) {
        if re.is_match(file_path.to_str().unwrap()) {
            let mut context = WorkflowExecutionContext::new(file_path, self.config.panic_handler_on_error);
            for action in &self.config.actions {
                let action_succeeded = action.run(&mut context);
                if !action_succeeded {
                    break;
                }
            }
        }
    }

    fn on_startup(&self, path: &PathBuf, re: &Regex) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            self.handle(&entry.path(), re);
        }
        println!("Ended startup phase");
    }

    fn on_watch(&self, watcher_rx: Receiver<Result<notify::Event, notify::Error>>, re: &Regex) {
        for result in watcher_rx {
            match result {
                Ok(event) => {
                    if self.config.event.is_handled_event(&event.kind) {
                        println!("Event to handle - {:?}", &event.kind);
                        let event_file_path = event.paths.first().unwrap();
                        self.handle(event_file_path, re);
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
        let re = Regex::new(&self.config.event.naming_regex_match).unwrap();
        if self.config.apply_on_startup {
            self.on_startup(path, &re);
        }
        self.on_watch(watcher_rx, &re);
        println!("Ending watch");
    }
}