use std::path::{Path, PathBuf};

use generated_types::TraceHandlerResponse;
use crate::{pipeline_config::PipelineConfig, pipeline_context_input::PipelineContextInput};

type OutputTraceSender = tokio::sync::broadcast::Sender<Result<TraceHandlerResponse, tonic::Status>>;

pub struct PipelineExecutionContext<'a> {
    pub config: PipelineConfig,
    pub event_file_path: PathBuf,
    pub action_file_path: Option<PathBuf>,
    pub trace_tx: &'a OutputTraceSender
}

impl<'a> PipelineExecutionContext<'a> {
    pub fn new<T>(event_file_path: T, config: PipelineConfig, trace_tx: &'a OutputTraceSender) -> Self 
    where 
    T: AsRef<Path> { 
        Self { 
            config,
            event_file_path: event_file_path.as_ref().to_path_buf(),
            action_file_path: Option::None,
            trace_tx
        } 
    }

    pub fn get_input(&self, input: PipelineContextInput) -> Option<PathBuf> {
        match input {
            PipelineContextInput::EventFilePath => Some(self.event_file_path.clone()),
            PipelineContextInput::ActionFilePath => self.action_file_path.clone()
        }
    }

    pub fn log<T>(&self, action: Option<String>, msg: T) 
    where
    T: AsRef<str> {
        tracing::info!("{}", msg.as_ref());
        self.send_trace_message(action, msg);
    }

    pub fn handle_error<T>(&self, action: Option<String>, msg: T) -> bool
    where 
    T: AsRef<str> {
        tracing::error!("{}", msg.as_ref());
        self.send_trace_message(action, msg.as_ref());
        if self.config.panic_handler_on_error {
            panic!("{}", msg.as_ref());
        }
        return false;
    }

    fn send_trace_message<T>(&self, action: Option<String>, msg: T) 
    where
    T: AsRef<str> {
        let _ = self.trace_tx.send(Ok(TraceHandlerResponse {
            directory_path: self.event_file_path.parent().unwrap().to_str().unwrap().to_string(),
            action,
            message: msg.as_ref().to_string(),
        }));
    }
}
