use std::path::{Path, PathBuf};

use crate::workflow_context_input::WorkflowContextInput;

pub struct WorkflowExecutionContext {
    pub panic_handler_on_error: bool,
    pub event_file_path: PathBuf,
    pub action_file_path: Option<PathBuf>,
}

impl WorkflowExecutionContext {
    pub fn new<T>(event_file_path: T, panic_handler_on_error: bool) -> Self 
    where 
    T: AsRef<Path> { 
        Self { 
            panic_handler_on_error,
            event_file_path: event_file_path.as_ref().to_path_buf(),
            action_file_path: Option::None,
        } 
    }

    pub fn get_input(&self, input: WorkflowContextInput) -> Option<PathBuf> {
        match input {
            WorkflowContextInput::EventFilePath => Some(self.event_file_path.clone()),
            WorkflowContextInput::ActionFilePath => self.action_file_path.clone()
        }
    }

    pub fn handle_error<T>(&self, msg: T) -> bool
    where 
    T: AsRef<str> {
        if self.panic_handler_on_error {
            panic!("{}", msg.as_ref());
        }
        else {
            println!("{}", msg.as_ref());
            return false;
        }
    }
}