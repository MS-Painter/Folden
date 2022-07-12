use std::{
    ffi::OsStr,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::{construct_working_dir, PipelineAction};
use crate::{
    pipeline_context_input::PipelineContextInput,
    pipeline_execution_context::PipelineExecutionContext,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoveToDir {
    pub input: PipelineContextInput,
    pub directory_path: PathBuf,
    pub requires_directory_exists: bool,
    pub replace_older_files: bool,
    pub keep_input_file_intact: bool,
    pub datetime_formatting: bool,
    pub must_succeed: bool,
}

impl MoveToDir {
    fn ensure_dir_exists(
        &self,
        context: &mut PipelineExecutionContext,
        working_dir_path: &Path,
    ) -> bool {
        if !working_dir_path.is_dir() {
            if self.requires_directory_exists {
                return context.handle_error("Directory required to exist");
            } else {
                fs::create_dir_all(&working_dir_path).unwrap();
                return true;
            }
        }
        true
    }

    fn apply(
        &self,
        context: &mut PipelineExecutionContext,
        working_dir_path: &Path,
        input_path: &Path,
        input_file_name: &OsStr,
    ) -> bool {
        if !self.ensure_dir_exists(context, working_dir_path) {
            return false;
        }
        let mut new_file_path = PathBuf::from(working_dir_path);
        new_file_path.push(input_file_name);
        if new_file_path.is_file() && !self.replace_older_files {
            context.handle_error("Can't replace older file")
        } else {
            match fs::copy(&input_path, &new_file_path) {
                Ok(_) => {
                    context.log("Copied file");
                    if self.keep_input_file_intact {
                        context.action_file_path = Some(new_file_path);
                        true
                    } else {
                        match fs::remove_file(input_path) {
                            Ok(_) => {
                                context.log("Deleted original file");
                                context.action_file_path = Some(new_file_path);
                                true
                            }
                            Err(err) => context.handle_error(format!("{}", err)),
                        }
                    }
                }
                Err(err) => context.handle_error(format!("{:?}", err)),
            }
        }
    }
}

impl Default for MoveToDir {
    fn default() -> Self {
        Self {
            input: PipelineContextInput::EventFilePath,
            directory_path: PathBuf::from("output_dir_path"),
            requires_directory_exists: false,
            replace_older_files: true,
            keep_input_file_intact: false,
            datetime_formatting: true,
            must_succeed: true,
        }
    }
}

impl PipelineAction for MoveToDir {
    fn run(&self, context: &mut PipelineExecutionContext) -> bool {
        match context.get_input(self.input) {
            Some(input_path) => match input_path.file_name() {
                Some(input_file_name) => {
                    let output_directory_path = if self.datetime_formatting {
                        PathBuf::from(Self::format_datetime(
                            &self.directory_path.to_string_lossy(),
                        ))
                    } else {
                        self.directory_path.to_path_buf()
                    };
                    let working_dir_path =
                        construct_working_dir(&input_path, &output_directory_path);
                    match working_dir_path.canonicalize() {
                        Ok(working_dir_path) => {
                            self.apply(context, &working_dir_path, &input_path, input_file_name)
                        }
                        Err(err) => match err.kind() {
                            ErrorKind::NotFound => {
                                if !self.ensure_dir_exists(context, &working_dir_path) {
                                    return false;
                                }
                                self.apply(
                                    context,
                                    &working_dir_path.canonicalize().unwrap(),
                                    &input_path,
                                    input_file_name,
                                )
                            }
                            _ => context.handle_error(format!("{:?}", err)),
                        },
                    }
                }
                None => context.handle_error("Path can't be parsed as file"),
            },
            None => context.handle_error("Input doesn't contain value"),
        }
    }

    fn must_succeed(&self) -> bool {
        self.must_succeed
    }
}
