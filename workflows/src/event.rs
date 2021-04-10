use std::io;

use clap::Values;
use notify::EventKind;
use itertools::Itertools;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowEvent {
    pub events: Vec<String>, // Can flag multiple events in the config to initiate the workflow against.
    pub naming_regex_match: String,
}

impl WorkflowEvent {
    fn get_event_kind(name: &str) -> Result<EventKind, io::Error> {
        match name.to_lowercase().as_str() {
            "access" => Ok(EventKind::Access(notify::event::AccessKind::Any)),
            "create" => Ok(EventKind::Create(notify::event::CreateKind::Any)),
            "modify" => Ok(EventKind::Modify(notify::event::ModifyKind::Any)),
            "remove" => Ok(EventKind::Remove(notify::event::RemoveKind::Any)),
            _ => Err(io::Error::new(io::ErrorKind::Other, "oh no!")),
        }
    }

    pub fn is_handled_event(&self, kind: &EventKind) -> bool {
        for event_name in &self.events {
            match WorkflowEvent::get_event_kind(event_name) {
                Ok(handled_kind) => {
                    if &handled_kind == kind {
                        return true;
                    }
                }
                Err(_) => {}
            }
        }
        return false;
    }
}

impl From<Values<'_>> for WorkflowEvent {
    fn from(events: Values) -> Self {
        Self {
            events: events.map(|event| event.to_string()).unique().collect(),
            naming_regex_match: String::from("*"),
        }
    }
}