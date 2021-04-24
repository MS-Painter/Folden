use std::fs;
use std::path::PathBuf;

use tracing;
use regex::Regex;
use crossbeam::channel::Receiver;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::actions::WorkflowAction;
use crate::pipeline_config::PipelineConfig;
use crate::pipeline_execution_context::PipelineExecutionContext;

pub struct PipelineHandler {
    pub config: PipelineConfig,
    pub naming_regex: Option<Regex>,
}

impl PipelineHandler {
    pub fn new(config: PipelineConfig) -> Self {
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
        let mut context = PipelineExecutionContext::new(file_path, self.config.clone());
        for action in &self.config.actions {
            let action_succeeded = action.run(&mut context);
            if !action_succeeded {
                break;
            }
        }
    }

    fn apply_on_existing_files(&self, path: &PathBuf) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let metadata = entry.metadata().unwrap();
            if metadata.is_file(){
                self.handle(&entry.path());
            }
        }
    }

    fn on_watch(&self, watcher_rx: Receiver<Result<notify::Event, notify::Error>>) {
        for result in watcher_rx {
            match result {
                Ok(event) => {
                    if self.config.event.is_handled_event(&event.kind) {
                        tracing::debug!("Event to handle - {:?}", &event.kind);
                        let event_file_path = event.paths.first().unwrap();
                        self.handle(event_file_path);
                    }
                }
                Err(error) => {
                    tracing::warn!("Watcher error - {:?}", error);
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
        if self.config.apply_on_startup_on_existing_files {
            self.apply_on_existing_files(path);
            tracing::info!("Ended startup phase");
        }
        self.on_watch(rx);
        tracing::info!("Ending watch");
    }
}