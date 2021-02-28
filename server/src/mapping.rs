use std::{collections::HashMap, convert::TryFrom, fs, io::ErrorKind as IOErrorKind, path::PathBuf, sync::Arc, thread};

use crossbeam::channel::Sender;
use serde::{Serialize, Deserialize};
use notify::{Error, ErrorKind as NotifyErrorKind, Event, EventKind, RecommendedWatcher, Watcher};

use crate::config::{Config, MappingStatusStrategy};
use folder_handler::{Handler, handlers_json::HandlersJson};
use generated_types::{HandlerStateResponse, HandlerStatus, HandlerSummary};

// Mapping data used to handle known directories to handle
// If a handler thread has ceased isn't known at realtime rather will be verified via channel whenever needed to check given a client request

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mapping {
    pub directory_mapping: HashMap<String, HandlerMapping> // Hash map key binds to directory path
}

impl Mapping {
    pub fn save(&self, mapping_status_strategy: &MappingStatusStrategy, mapping_state_path: &PathBuf) -> Result<(), std::io::Error> {
        match mapping_status_strategy {
            MappingStatusStrategy::None => Err(std::io::Error::new(IOErrorKind::Other, "Not allowed current in config state")),
            _ => {
                let mapping_data: Vec<u8> = self.into();
                fs::write(mapping_state_path, mapping_data)
            }
        }
    }

    pub fn start_handler(&mut self, handlers_json: Arc<HandlersJson>, directory_path: &str, handler_mapping: &HandlerMapping) -> HandlerStateResponse {
        match handler_mapping.status() {
            HandlerStatus::Dead => {
                let handler_type_name = handler_mapping.handler_type_name.clone();
                let handler_config_path = handler_mapping.handler_config_path.clone();
                self.spawn_handler_thread(handlers_json, directory_path.to_string(), handler_type_name, handler_config_path);
                HandlerStateResponse {
                    state: HandlerStatus::Live as i32,
                    message: String::from("Handler started"),
                }
            }
            HandlerStatus::Live => HandlerStateResponse {
                state: HandlerStatus::Live as i32,
                message: String::from("Handler already up"),
            },
        }
    }
    
    pub fn spawn_handler_thread(&mut self, handlers_json: Arc<HandlersJson>, directory_path: String, handler_type_name: String, handler_config_path: String) {
        match handlers_json.get_handler_by_name(&handler_type_name) {
            Ok(mut handler) => {
                let path = PathBuf::from(directory_path.clone());
                let config_path = PathBuf::from(handler_config_path.clone());
                let (tx, rx) = crossbeam::channel::unbounded();
                let thread_tx = tx.clone();
                let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| thread_tx.send(res).unwrap()).unwrap();
                thread::spawn(move || {
                    let _ = watcher.watch(path.clone(), notify::RecursiveMode::NonRecursive);
                    handler.watch(&path, &config_path, rx);
                });            
                // Insert or update the value of the current handled directory
                self.directory_mapping.insert(directory_path, HandlerMapping {
                    watcher_tx: Option::Some(tx),
                    handler_type_name,
                    handler_config_path,
                });
            },
            Err(e) => panic!(e)
        }
    }

    pub async fn stop_handler(&mut self, config: &Config, directory_path: &str, handler_mapping: &HandlerMapping, remove: bool) -> HandlerStateResponse {
        let handler_type_name = handler_mapping.handler_type_name.clone();
        let handler_config_path = handler_mapping.handler_config_path.clone();

        match handler_mapping.status() {
            HandlerStatus::Dead => {
                let mut message = String::from("Handler already stopped");
                if remove {
                    self.directory_mapping.remove(directory_path);
                    message.push_str(" & removed");
                    let _result = self.save(&config.mapping_status_strategy, &config.mapping_state_path);
                }
                else {
                    self.directory_mapping.insert(directory_path.to_owned(), HandlerMapping {
                        watcher_tx: Option::None,
                        handler_type_name,
                        handler_config_path,
                    });
                }
                HandlerStateResponse {
                    state: HandlerStatus::Dead as i32,
                    message,
                }
            }
            HandlerStatus::Live => {
                match handler_mapping.stop_handler_thread() {
                    Ok(mut message) => {
                        if remove {
                            self.directory_mapping.remove(directory_path);
                            message.push_str(" & removed");
                            let _result = self.save(&config.mapping_status_strategy, &config.mapping_state_path);
                        }
                        else {
                            self.directory_mapping.insert(directory_path.to_owned(), HandlerMapping {
                                watcher_tx: Option::None,
                                handler_type_name,
                                handler_config_path,
                            });
                        }
                        HandlerStateResponse {
                            state: HandlerStatus::Dead as i32,
                            message,
                        }
                    }
                    Err(message) => {
                        HandlerStateResponse {
                            state: HandlerStatus::Live as i32,
                            message,
                        }
                    }
                }
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
    pub handler_type_name: String,
    pub handler_config_path: String,
}

impl HandlerMapping {
    pub fn status(&self) -> HandlerStatus {
        match self.watcher_tx.clone() {
            Some(tx) => {
                match tx.send(Ok(Event::new(EventKind::Other))) {
                    Ok(_) => HandlerStatus::Live,
                    Err(_) => HandlerStatus::Dead
                }
            }
            None => HandlerStatus::Dead
        }
    }

    pub fn summary(&self) -> HandlerSummary {
        let state = HandlerSummary {
            state: self.status() as i32,
            type_name: self.handler_type_name.clone(),
            config_path: self.handler_config_path.clone(),
        };
        state
    }

    pub fn stop_handler_thread(&self) -> Result<String, String> {
        match self.watcher_tx.clone().unwrap().send(Err(Error::new(NotifyErrorKind::WatchNotFound))) {
            Ok(_) => {
                Ok(String::from("Handler stopped"))
            }
            Err(error) => {
                let mut message = String::from("Failed to stop handler\nError: ");
                message.push_str(error.to_string().as_str());
                Err(message)
            }
        }
    }
}