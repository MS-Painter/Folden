use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{pipeline_config::PipelineConfig, pipeline_context_input::PipelineContextInput};
use generated_types::TraceHandlerResponse;

type OutputTraceSender =
    Arc<tokio::sync::broadcast::Sender<Result<TraceHandlerResponse, tonic::Status>>>;

pub struct PipelineExecutionContext {
    pub config: PipelineConfig,
    pub event_file_path: PathBuf,
    pub action_file_path: Option<PathBuf>,
    pub trace_tx: OutputTraceSender,
    pub action_name: Option<String>,
}

impl<'a> PipelineExecutionContext {
    pub fn new<T>(event_file_path: T, config: PipelineConfig, trace_tx: OutputTraceSender) -> Self
    where
        T: AsRef<Path>,
    {
        Self {
            config,
            event_file_path: event_file_path.as_ref().to_path_buf(),
            action_file_path: None,
            trace_tx,
            action_name: None,
        }
    }

    pub fn get_input(&self, input: PipelineContextInput) -> Option<PathBuf> {
        match input {
            PipelineContextInput::EventFilePath => Some(self.event_file_path.clone()),
            PipelineContextInput::ActionFilePath => self.action_file_path.clone(),
        }
    }

    pub fn log<T>(&self, msg: T)
    where
        T: AsRef<str>,
    {
        tracing::info!("{}", msg.as_ref());
        self.send_trace_message(msg);
    }

    pub fn handle_error<T>(&self, msg: T) -> bool
    where
        T: AsRef<str>,
    {
        tracing::error!("{}", msg.as_ref());
        self.send_trace_message(msg.as_ref());
        if self.config.panic_handler_on_error {
            panic!("{}", msg.as_ref());
        }
        false
    }

    fn send_trace_message<T>(&self, msg: T)
    where
        T: AsRef<str>,
    {
        let _ = self.trace_tx.send(Ok(TraceHandlerResponse {
            directory_path: self
                .event_file_path
                .parent()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            action: self.action_name.to_owned(),
            message: msg.as_ref().to_string(),
        }));
    }
}
