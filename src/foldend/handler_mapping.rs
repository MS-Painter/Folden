use std::convert::TryFrom;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use crossbeam::channel::Sender;
use notify::ErrorKind as NotifyErrorKind;
use notify::RecommendedWatcher;
use notify::Watcher;
use notify::{Event, EventKind};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use generated_types::{HandlerSummary, ModifyHandlerRequest};
use pipelines::pipeline_config::PipelineConfig;
use pipelines::pipeline_handler::PipelineHandler;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandlerMapping {
    #[serde(skip)]
    pub watcher_tx: Option<Sender<Result<Event, notify::Error>>>, // Channel sender providing thread health and allowing manual thread shutdown
    pub handler_config_path: String,
    pub is_auto_startup: bool,
    pub description: String,
}

impl HandlerMapping {
    pub fn new(handler_config_path: String, is_auto_startup: bool, description: String) -> Self {
        Self {
            watcher_tx: None,
            handler_config_path,
            is_auto_startup,
            description,
        }
    }

    pub fn is_alive(&self) -> bool {
        match self.watcher_tx.clone() {
            Some(tx) => tx.send(Ok(Event::new(EventKind::Other))).is_ok(),
            None => false,
        }
    }

    pub fn summary(&self) -> HandlerSummary {
        HandlerSummary {
            is_alive: self.is_alive(),
            config_path: self.handler_config_path.clone(),
            is_auto_startup: self.is_auto_startup,
            description: self.description.to_owned(),
        }
    }

    pub fn spawn_handler_thread(
        &mut self,
        directory_path: String,
        trace_tx: Arc<
            broadcast::Sender<Result<generated_types::TraceHandlerResponse, tonic::Status>>,
        >,
    ) -> Result<(), String> {
        let path = PathBuf::from(directory_path);
        let config_path = PathBuf::from(&self.handler_config_path);
        match fs::read(&config_path) {
            Ok(data) => {
                match PipelineConfig::try_from(data) {
                    Ok(config) => {
                        let (events_tx, events_rx) = crossbeam::channel::unbounded();
                        let events_thread_tx = events_tx.clone();
                        let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| events_thread_tx.send(res).unwrap()).unwrap();
                        let _ = watcher.configure(notify::Config::PreciseEvents(true));
                        thread::spawn(move || {
                            let mut handler = PipelineHandler::new(config, trace_tx);
                            handler.watch(&path, watcher, events_rx);
                        });
                        self.watcher_tx = Option::Some(events_tx);
                        Ok(())
                    }
                    Err(err) => Err(format!("Pipeline config parsing failure.\nPath: {:?}\nError: {:?}", config_path, err))
                }
            }
            Err(err) => {
                Err(format!("Pipeline file read failure.\nMake sure the file is at the registered path\nPath: {:?}\nError: {:?}", config_path, err))
            }
        }
    }

    pub fn stop_handler_thread(&self) -> Result<String, String> {
        match self
            .watcher_tx
            .clone()
            .unwrap()
            .send(Err(notify::Error::new(NotifyErrorKind::WatchNotFound)))
        {
            Ok(_) => Ok(String::from("Handler stopped")),
            Err(error) => Err(format!("Failed to stop handler\nError: {:?}", error)),
        }
    }

    pub fn modify(&mut self, request: &ModifyHandlerRequest) {
        if let Some(is_auto_startup) = request.is_auto_startup {
            self.is_auto_startup = is_auto_startup;
        }
        if let Some(ref description) = request.modify_description {
            self.description = description.to_string();
        }
    }
}
