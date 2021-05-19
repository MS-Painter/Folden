use std::{collections::HashMap, convert::TryFrom, fs, path::PathBuf, result::Result, sync::Arc, thread};

use tokio::sync::{Mutex, mpsc};
use serde::{Serialize, Deserialize};
use notify::{RecommendedWatcher, Watcher};

use generated_types::HandlerStateResponse;
use crate::{config::Config, handler_mapping::HandlerMapping};
use pipelines::{pipeline_config::PipelineConfig, pipeline_handler::PipelineHandler};

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

    pub fn get_live_handlers<'a>(&'a self) -> impl Iterator<Item = (&'a String, &'a HandlerMapping)> {
        self.directory_mapping.iter().filter(|(_dir, mapping)| mapping.is_alive())
    }

    pub fn start_handler(&mut self, directory_path: &str, handler_mapping: &mut HandlerMapping) -> HandlerStateResponse {
        if handler_mapping.is_alive() {
            HandlerStateResponse {
                is_alive: true,
                message: String::from("Handler already up"),
            }
        }
        else {
            match self.spawn_handler_thread(directory_path.to_string(), handler_mapping) {
                Ok(_) => HandlerStateResponse {
                    is_alive: true,
                    message: String::from("Started handler"),
                },
                Err(err) => HandlerStateResponse {
                    is_alive: false,
                    message: format!("Failed to start handler.\nError: {}", err),
                }
            }
        }
    }
    
    pub fn spawn_handler_thread(&mut self, directory_path: String, handler_mapping: &mut HandlerMapping) -> Result<(), String> {
        let path = PathBuf::from(directory_path.clone());
        let config_path = PathBuf::from(&handler_mapping.handler_config_path);
        match fs::read(&config_path) {
            Ok(data) => {
                match PipelineConfig::try_from(data) {
                    Ok(config) => {
                        let (events_tx, events_rx) = crossbeam::channel::unbounded();
                        let (trace_tx, trace_rx) = mpsc::channel(3);
                        let thread_tx = events_tx.clone();
                        let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| thread_tx.send(res).unwrap()).unwrap();
                        let _ = watcher.configure(notify::Config::PreciseEvents(true));
                        thread::spawn(move || {
                            let mut handler = PipelineHandler::new(config);
                            handler.watch(&path, watcher, events_rx, trace_tx);
                        });            
                        // Insert or update the value of the current handled directory
                        handler_mapping.watcher_tx = Option::Some(events_tx);
                        handler_mapping.watcher_rx = Option::Some(Arc::new(Mutex::new(trace_rx)));
                        self.directory_mapping.insert(directory_path, handler_mapping.to_owned());
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
