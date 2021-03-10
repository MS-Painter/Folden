use notify::EventKind;
use chrono::{DateTime, Local};

pub struct WorkflowEvent {
    pub io_events: Vec<EventKind>, // Can flag multiple events in the config to initiate the workflow against.
    pub naming_regex_match: String,
    pub from_date_created: DateTime<Local>,
}