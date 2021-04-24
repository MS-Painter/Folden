use std::path::{Path, PathBuf};

use crate::{pipeline_config::PipelineConfig, pipeline_context_input::PipelineContextInput};

pub struct WorkflowExecutionContext {
    pub config: PipelineConfig,
    pub event_file_path: PathBuf,
    pub action_file_path: Option<PathBuf>,
}

impl WorkflowExecutionContext {
    pub fn new<T>(event_file_path: T, config: PipelineConfig) -> Self 
    where 
    T: AsRef<Path> { 
        Self { 
            config,
            event_file_path: event_file_path.as_ref().to_path_buf(),
            action_file_path: Option::None,
        } 
    }

    pub fn get_input(&self, input: PipelineContextInput) -> Option<PathBuf> {
        match input {
            PipelineContextInput::EventFilePath => Some(self.event_file_path.clone()),
            PipelineContextInput::ActionFilePath => self.action_file_path.clone()
        }
    }

    pub fn handle_error<T>(&self, msg: T) -> bool
    where 
    T: AsRef<str> {
        if self.config.panic_handler_on_error {
            tracing::error!("{}", msg.as_ref());
            panic!("{}", msg.as_ref());
        }
        else {
            tracing::error!("{}", msg.as_ref());
            return false;
        }
    }
}