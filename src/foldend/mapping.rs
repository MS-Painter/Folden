use std::{collections::HashMap, convert::TryFrom, fs, path::PathBuf, thread};

use crossbeam::channel::Sender;
use serde::{Serialize, Deserialize};
use notify::{Error, ErrorKind as NotifyErrorKind, Event, EventKind, RecommendedWatcher, Watcher};

use crate::config::Config;
use generated_types::{HandlerStateResponse, HandlerSummary, ModifyHandlerRequest};
use workflows::{workflow_config::WorkflowConfig, workflow_handler::WorkflowHandler};

// Mapping data used to handle known directories to handle
// If a handler thread has ceased isn't known at realtime rather will be verified via channel whenever needed to check given a client request

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mapping {
    pub directory_mapping: HashMap<String, HandlerMapping> // Hash map key binds to directory path
}

impl Mapping {
    pub fn save(&self, mapping_state_path: &PathBuf) -> Result<(), std::io::Error> {
        let mapping_data: Vec<u8> = self.into();
        fs::write(mapping_state_path, mapping_data)
    }

    pub fn start_handler(&mut self, directory_path: &str, handler_mapping: &mut HandlerMapping) -> HandlerStateResponse {
        if handler_mapping.is_alive() {
            HandlerStateResponse {
                is_alive: true,
                message: String::from("Handler already up"),
            }
        }
        else {
            self.spawn_handler_thread(directory_path.to_string(), handler_mapping);
            HandlerStateResponse {
                is_alive: true,
                message: String::from("Handler started"),
            }
        }
    }
    
    pub fn spawn_handler_thread(&mut self, directory_path: String, handler_mapping: &mut HandlerMapping) {
        let path = PathBuf::from(directory_path.clone());
        let config_path = PathBuf::from(&handler_mapping.handler_config_path);
        let config = WorkflowConfig::from_config(&config_path);
        let (tx, rx) = crossbeam::channel::unbounded();
        let thread_tx = tx.clone();
        let watcher: RecommendedWatcher = Watcher::new_immediate(move |res| thread_tx.send(res).unwrap()).unwrap();
        thread::spawn(move || {
            let mut handler = WorkflowHandler::new(config);
            handler.watch(&path, watcher, rx);
        });            
        // Insert or update the value of the current handled directory
        match self.directory_mapping.get_mut(&directory_path) {
            Some(handler_mapping) => {
                handler_mapping.watcher_tx = Option::Some(tx);
            }
            None => {
                handler_mapping.watcher_tx = Option::Some(tx);
                self.directory_mapping.insert(directory_path, handler_mapping.to_owned());
            }
        }
    }

    pub async fn stop_handler(&mut self, config: &Config, directory_path: &str, handler_mapping: &mut HandlerMapping, remove: bool) -> HandlerStateResponse {
        if handler_mapping.is_alive() {
            match handler_mapping.stop_handler_thread() {
                Ok(mut message) => {
                    if remove {
                        self.directory_mapping.remove(directory_path);
                        message.push_str(" & removed");
                        let _result = self.save(&config.mapping_state_path);
                    }
                    else {
                        handler_mapping.watcher_tx = None;
                    }
                    HandlerStateResponse {
                        is_alive: false,
                        message,
                    }
                }
                Err(message) => HandlerStateResponse {
                    is_alive: true,
                    message,
                }
            }
        }
        else {
            let mut message = String::from("Handler already stopped");
            if remove {
                self.directory_mapping.remove(directory_path);
                message.push_str(" & removed");
                let _result = self.save(&config.mapping_state_path);
            }
            else {
                handler_mapping.watcher_tx = None;
            }
            HandlerStateResponse {
                is_alive: false,
                message,
            }
        }
    }
}

impl TryFrom<Vec<u8>> for Mapping {
    type Error = &'static str;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        match toml::from_slice(&bytes) {
            Ok(mapping) => Ok(mapping), 
            Err(_) => Err("Couldn't deserialize data to mapping"),
        }
    }
}

impl Into<Vec<u8>> for Mapping {
    fn into(self) -> Vec<u8> {
        toml::to_vec(&self).unwrap()
    }
}

impl Into<Vec<u8>> for &Mapping {
    fn into(self) -> Vec<u8> {
        toml::to_vec(self).unwrap()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandlerMapping {
    #[serde(skip)]
    pub watcher_tx: Option<Sender<Result<Event, Error>>>, // Channel sender providing thread health and allowing manual thread shutdown
    pub handler_config_path: String,
    pub is_auto_startup: bool,
    pub description: String,
}

impl HandlerMapping {
    pub fn is_alive(&self) -> bool {
        match self.watcher_tx.clone() {
            Some(tx) => tx.send(Ok(Event::new(EventKind::Other))).is_ok(),
            None => false
        }
    }

    pub fn summary(&self) -> HandlerSummary {
        let state = HandlerSummary {
            is_alive: self.is_alive(),
            config_path: self.handler_config_path.clone(),
            is_auto_startup: self.is_auto_startup,
            description: self.description.to_owned(),
        };
        state
    }

    pub fn stop_handler_thread(&self) -> Result<String, String> {
        match self.watcher_tx.clone().unwrap().send(Err(Error::new(NotifyErrorKind::WatchNotFound))) {
            Ok(_) => Ok(String::from("Handler stopped")),
            Err(error) => Err(format!("Failed to stop handler\nError: {:?}", error))
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