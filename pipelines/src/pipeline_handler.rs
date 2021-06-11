use std::fs;
use std::sync::Arc;
use std::path::PathBuf;

use tracing;
use regex::Regex;
use crossbeam::channel::Receiver;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::actions::PipelineAction;
use generated_types::TraceHandlerResponse;
use crate::pipeline_config::PipelineConfig;
use crate::pipeline_execution_context::PipelineExecutionContext;

type OutputTraceSender = Arc<tokio::sync::broadcast::Sender<Result<TraceHandlerResponse, tonic::Status>>>;

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

    fn handle(&self, file_path: &PathBuf, trace_tx: &OutputTraceSender) {
        match &self.naming_regex {
            Some(naming_regex) => {
                if naming_regex.is_match(file_path.to_str().unwrap()) {
                    self.execute_pipeline(file_path, trace_tx);
                }
            }
            None => self.execute_pipeline(file_path, trace_tx)
        }
    }

    fn execute_pipeline(&self, file_path: &PathBuf, trace_tx: &OutputTraceSender) {
        let mut context = PipelineExecutionContext::new(file_path, self.config.clone(), trace_tx);
        for action in &self.config.actions {
            let action_succeeded = action.run(&mut context);
            if !action_succeeded {
                break;
            }
        }
    }

    fn apply_on_existing_files(&self, path: &PathBuf, trace_tx: &OutputTraceSender) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let metadata = entry.metadata().unwrap();
            if metadata.is_file(){
                self.handle(&entry.path(), trace_tx);
            }
        }
    }

    fn on_watch(&self, watcher_rx: Receiver<Result<notify::Event, notify::Error>>, trace_tx: &OutputTraceSender) {
        for result in watcher_rx {
            match result {
                Ok(event) => {
                    if self.config.event.is_handled_event(&event.kind) {
                        tracing::debug!("Event to handle - {:?}", &event.kind);
                        let event_file_path = event.paths.first().unwrap();
                        self.handle(event_file_path, trace_tx);
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

    pub fn watch(&mut self, path: &PathBuf, mut watcher: RecommendedWatcher, 
        events_rx: Receiver<Result<notify::Event, notify::Error>>, trace_tx: OutputTraceSender) {
        let recursive_mode = if self.config.watch_recursive {RecursiveMode::Recursive} else {RecursiveMode::NonRecursive};
        watcher.watch(path.clone(), recursive_mode).unwrap();
        if self.config.apply_on_startup_on_existing_files {
            self.apply_on_existing_files(path, &trace_tx);
            tracing::info!("Ended startup phase");
        }
        self.on_watch(events_rx, &trace_tx);
        tracing::info!("Ending watch");
    }
}