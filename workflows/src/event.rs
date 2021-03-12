use clap::Values;
use notify::EventKind;
use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowEvent {
    pub events: Vec<String>, // Can flag multiple events in the config to initiate the workflow against.
    pub naming_regex_match: String,
    #[serde(with = "custom_datetime_format")]
    pub from_date_created: DateTime<Local>,
}

impl WorkflowEvent {
    fn get_event_name(kind: EventKind) -> &'static str {
        match kind {
            EventKind::Access(_) => "access",
            EventKind::Create(_) => "create",
            EventKind::Modify(_) => "modify",
            EventKind::Remove(_) => "remove",
            _ => "",
        }
    }

    fn get_event_kind(name: &str) -> EventKind {
        match name.to_lowercase().as_str() {
            "access" => EventKind::Access(notify::event::AccessKind::Any),
            "create" => EventKind::Create(notify::event::CreateKind::Any),
            "modify" => EventKind::Modify(notify::event::ModifyKind::Any),
            "remove" => EventKind::Remove(notify::event::RemoveKind::Any),
            _ => EventKind::Other
        }
    }
}

impl From<Values<'_>> for WorkflowEvent {
    fn from(events: Values) -> Self {
        Self {
            events: events.map(|event| event.to_string()).collect(),
            naming_regex_match: String::from("*"),
            from_date_created: Local::now(),
        }
    }
}

mod custom_datetime_format {
    use chrono::{DateTime, Local, TimeZone};
    use serde::{self, Deserialize, Serializer, Deserializer};
    
    const FORMAT: &'static str = "%d-%m-%Y %H:%M:%S";

    pub fn serialize<S>(date: &DateTime<Local>, serializer: S,) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D,) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Local.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}