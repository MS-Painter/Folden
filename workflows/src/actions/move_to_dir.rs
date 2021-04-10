use std::{fs, path::PathBuf};

use serde::{Serialize, Deserialize};

use super::WorkflowAction;
use crate::workflow_execution_context::WorkflowExecutionContext;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoveToDir {
    pub directory_name: PathBuf,
    pub requires_directory_exists: bool,
    pub replace_older_files: bool,
}

impl Default for MoveToDir {
    fn default() -> Self {
        Self {
            directory_name: PathBuf::from("new_dir_name"),
            requires_directory_exists: false,
            replace_older_files: true,
        }
    }
}

impl WorkflowAction for MoveToDir {
    fn run(&self, context: &mut WorkflowExecutionContext) {
        match context.event_file_path.file_name() {
            Some(event_file_name) => {
                if !self.directory_name.is_dir() {
                    if self.requires_directory_exists {
                        panic!();
                    }
                    else {
                        fs::create_dir(&self.directory_name).unwrap();
                    }
                }
                let mut new_file_path = PathBuf::from(&self.directory_name);
                new_file_path.push(event_file_name);
                match fs::copy(&context.event_file_path, &new_file_path) {
                    Ok(_) => {
                        match fs::remove_file(event_file_name) {
                            Ok(_) => {}
                            Err(err) => {
                                println!("{}", err);
                            }
                        }
                    }
                    Err(err) => {
                        println!("{}", err);
                    }
                }
            }
            None => {}
        }
    }
}