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
    pub keep_input_file_intact: bool,
}

impl Default for MoveToDir {
    fn default() -> Self {
        Self {
            input: WorkflowContextInput::EventFilePath,
            directory_path: PathBuf::from("output_dir_path"),
            requires_directory_exists: false,
            replace_older_files: true,
            keep_input_file_intact: false,
        }
    }
}

impl WorkflowAction for MoveToDir {
    fn run(&self, context: &mut WorkflowExecutionContext) -> bool {
        match context.get_input(self.input) {
            Some(input_path) => {
                match input_path.file_name() {
                    Some(input_file_name) => {
                        let working_dir_path = self.construct_working_dir(&input_path);
                        if !working_dir_path.is_dir() {
                            if self.requires_directory_exists {
                                return context.handle_error("Directory required to exist");
                            }
                            else {
                                fs::create_dir(&working_dir_path).unwrap();
                            }
                        }
                        let mut new_file_path = PathBuf::from(&working_dir_path);
                        new_file_path.push(input_file_name);
                        if new_file_path.is_file() && !self.replace_older_files {
                            return context.handle_error("Can't replace older file");
                        }
                        else {
                            match fs::copy(&input_path, &new_file_path) {
                                Ok(_) => {
                                    if self.keep_input_file_intact {
                                        context.action_file_path = Some(new_file_path);
                                        true
                                    }
                                    else {
                                        match fs::remove_file(input_path) {
                                            Ok(_) => {
                                                context.action_file_path = Some(new_file_path);
                                                true
                                            },
                                            Err(err) => context.handle_error(format!("{}", err))
                                        }
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

    fn construct_working_dir(&self, input_path: &PathBuf) -> PathBuf {
        let mut working_path = PathBuf::from(input_path.parent().unwrap());
        working_path.push(&self.directory_path); // If directory_path is absolute will replace the entire path
        working_path
    }
}