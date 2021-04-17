use std::fs;
use std::path::PathBuf;

use regex::Regex;
use crossbeam::channel::Receiver;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::actions::WorkflowAction;
use crate::workflow_config::WorkflowConfig;
use crate::workflow_execution_context::WorkflowExecutionContext;

pub struct WorkflowHandler {
    pub config: WorkflowConfig,
    pub naming_regex: Option<Regex>,
}

impl WorkflowHandler {
    pub fn new(config: WorkflowConfig) -> Self {
        match config.event.naming_regex_match.to_owned() {
            Some(naming_regex_match) => Self { 
                config,
                naming_regex: Some(Regex::new(&naming_regex_match).unwrap())
            },
            None => Self { 
                config, 
                naming_regex: None
            },
        }
    }

    fn handle(&self, file_path: &PathBuf) {
        match &self.naming_regex {
            Some(naming_regex) => {
                if naming_regex.is_match(file_path.to_str().unwrap()) {
                    self.execute_workflow(file_path);
                }
            }
            None => self.execute_workflow(file_path)
        }
    }

    fn execute_workflow(&self, file_path: &PathBuf) {
        let mut context = WorkflowExecutionContext::new(file_path, self.config.panic_handler_on_error);
        for action in &self.config.actions {
            let action_succeeded = action.run(&mut context);
            if !action_succeeded {
                break;
            }
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

    pub fn watch(&mut self, path: &PathBuf, mut watcher: RecommendedWatcher, rx: Receiver<Result<notify::Event, notify::Error>>) {
        let recursive_mode = if self.config.watch_recursive {RecursiveMode::Recursive} else {RecursiveMode::NonRecursive};
        watcher.watch(path.clone(), recursive_mode).unwrap();
        if self.config.apply_on_startup {
            self.on_startup(path);
        }
        self.on_watch(rx);
        println!("Ending watch");
    }
}