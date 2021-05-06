use notify::{Event, EventKind};
use crossbeam::channel::Sender;
use serde::{Serialize, Deserialize};
use notify::ErrorKind as NotifyErrorKind;

use generated_types::{HandlerSummary, ModifyHandlerRequest};

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
        match self.watcher_tx.clone().unwrap().send(Err(notify::Error::new(NotifyErrorKind::WatchNotFound))) {
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