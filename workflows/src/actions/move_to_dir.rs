use std::{fs, path::PathBuf};

use serde::{Serialize, Deserialize};

use super::WorkflowAction;
use crate::{workflow_context_input::WorkflowContextInput, workflow_execution_context::WorkflowExecutionContext};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoveToDir {
    pub input: WorkflowContextInput,
    pub directory_path: PathBuf,
    pub requires_directory_exists: bool,
    pub replace_older_files: bool,
}

impl Default for MoveToDir {
    fn default() -> Self {
        Self {
            directory_path: PathBuf::from("output_dir_path"),
            requires_directory_exists: false,
            replace_older_files: true,
            input: WorkflowContextInput::EventFilePath,
        }
    }
}

impl WorkflowAction for MoveToDir {
    fn run(&self, context: &mut WorkflowExecutionContext) -> bool {
        match context.get_input(self.input) {
            Some(input_path) => {
                match input_path.file_name() {
                    Some(input_file_name) => {
                        if !self.directory_path.is_dir() {
                            if self.requires_directory_exists {
                                return context.handle_error("Directory required to exist");
                            }
                            else {
                                fs::create_dir(&self.directory_path).unwrap();
                            }
                        }
                        let mut new_file_path = PathBuf::from(&self.directory_path);
                        new_file_path.push(input_file_name);
                        if new_file_path.is_file() && !self.replace_older_files {
                            return context.handle_error("Can't replace older file");
                        }
                        else {
                            match fs::copy(&input_path, &new_file_path) {
                                Ok(_) => {
                                    match fs::remove_file(event_file_name) {
                                        Ok(_) => {
                                            context.action_file_path = Some(new_file_path);
                                            true
                                        },
                                        Err(err) => context.handle_error(format!("{}", err))
                                    }
                                },
                                Err(err) => context.handle_error(format!("{}", err))
                            }
                        }
                    }
                    None => context.handle_error("Path can't be parsed as file")
                }
            }
            None => context.handle_error("Input doesn't contain value")
        }
    }
}