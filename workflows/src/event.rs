use clap::Values;
use notify::EventKind;
use itertools::Itertools;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowEvent {
    pub events: Vec<String>, // Can flag multiple events in the config to initiate the workflow against.
    pub naming_regex_match: Option<String>,
}

pub const EVENT_TYPES: [&str; 2] = ["create", "modify"];

impl WorkflowEvent {
    fn is_handled_event_kind(name: &str, kind: &EventKind) -> bool {
        match name.to_lowercase().as_str() {
            "create" => kind.is_create(),
            "modify" => kind.is_modify(),
            _ => false,
        }
    }

    pub fn is_handled_event(&self, kind: &EventKind) -> bool {
        for event_name in &self.events {
            if WorkflowEvent::is_handled_event_kind(event_name, kind) {
                return true;
            }
        }
        return false;
    }
}

impl From<Values<'_>> for WorkflowEvent {
    fn from(events: Values) -> Self {
        Self {
            events: events.map(|event| event.to_string()).unique().collect(),
            naming_regex_match: Some(String::from(".*")),
        }
    }
}

impl Default for WorkflowEvent {
    fn default() -> Self {
        Self {
            events: EVENT_TYPES.iter().map(|event| event.to_string()).collect(),
            naming_regex_match: Some(String::from(".*")),
        }
    }
}