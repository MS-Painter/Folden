use std::path::{Path, PathBuf};

pub struct WorkflowExecutionContext {
    pub event_file_path: PathBuf,
    pub action_file_path: Option<PathBuf>,
    pub action_file_data: Option<Vec<u8>>,
}

impl WorkflowExecutionContext {
    pub fn new<T>(event_file_path: T) -> Self 
    where 
    T: AsRef<Path> { 
        Self { 
            event_file_path: event_file_path.as_ref().to_path_buf(),
            action_file_path: Option::None,
            action_file_data: Option::None,
        } 
    }
}