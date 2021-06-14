use clap::Values;
use itertools::Itertools;
use notify::EventKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineEvent {
    pub events: Vec<String>, // Can flag multiple events in the config to initiate the pipeline against.
    pub naming_regex_match: Option<String>,
}

pub const EVENT_TYPES: [&str; 2] = ["create", "modify"];

impl PipelineEvent {
    fn is_handled_event_kind(name: &str, kind: &EventKind) -> bool {
        match name.to_lowercase().as_str() {
            "create" => kind.is_create(),
            "modify" => kind.is_modify(),
            _ => false,
        }
    }

    pub fn is_handled_event(&self, kind: &EventKind) -> bool {
        for event_name in &self.events {
            if PipelineEvent::is_handled_event_kind(event_name, kind) {
                return true;
            }
        }
        false
    }
}

impl From<Values<'_>> for PipelineEvent {
    fn from(events: Values) -> Self {
        Self {
            events: events.map(|event| event.to_string()).unique().collect(),
            naming_regex_match: Some(String::from(".*")),
        }
    }
}

impl Default for PipelineEvent {
    fn default() -> Self {
        Self {
            events: EVENT_TYPES.iter().map(|event| event.to_string()).collect(),
            naming_regex_match: Some(String::from(".*")),
        }
    }
}
