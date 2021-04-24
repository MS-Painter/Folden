use std::{borrow::Cow, path::PathBuf};

use chrono;
use regex::Regex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

mod run_cmd;
mod move_to_dir;
use self::{move_to_dir::MoveToDir, run_cmd::RunCmd};
use crate::pipeline_execution_context::PipelineExecutionContext;

pub const ACTION_TYPES: [&str; 2] = ["runcmd", "movetodir"];

pub trait PipelineAction {
    // Execute action. Returns if action deemed successful.
    fn run(&self, context: &mut PipelineExecutionContext) -> bool;
}

pub fn format_windows_directory_space_seperation<T>(text: T) -> String
where
T: AsRef<str> {
    lazy_static! {
        static ref WINDOWS_SPACE_SEPERATED_DIRECTORIES_RE: Regex = Regex::new(r"(?P<dir>\w+ \w+)").unwrap();
    }
    WINDOWS_SPACE_SEPERATED_DIRECTORIES_RE.replace_all(text.as_ref(), "\"$dir\"").to_string()
}

pub fn format_input(text: &String, input: Option<PathBuf>) -> Result<Cow<str>,()> {
    if let Some(input) = input {
        lazy_static! {
            static ref INPUT_RE: Regex = Regex::new(r"(\$input\$)").unwrap();
        }
        
        let input_replaced_text: Cow<str> = INPUT_RE.replace_all(text, input.to_string_lossy());
        if cfg!(windows) {
            let dir_space_handled_text = format_windows_directory_space_seperation(&input_replaced_text);
            return Ok(dir_space_handled_text.into())
        }
        return Ok(input_replaced_text.into())
    }
    Err(())
}

pub fn format_datetime<S>(text: S) -> String where S: AsRef<str> {
    chrono::Local::now().format(text.as_ref()).to_string()
}

pub fn construct_working_dir(input_path: &PathBuf, directory_path: &PathBuf) -> PathBuf {
    let mut working_path = PathBuf::from(input_path.parent().unwrap());
    working_path.push(directory_path); // If directory_path is absolute will replace the entire path
    working_path
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PipelineActions {
    MoveToDir(MoveToDir),
    RunCmd(RunCmd),
    None,
}

impl PipelineActions {
    pub fn defaults<'a, I>(actions: I) -> Vec<PipelineActions> 
    where I: Iterator<Item = &'a str> {
        actions.map(|action_name| {
            match action_name.to_lowercase().as_str() {
                "runcmd" => Self::RunCmd(RunCmd::default()),
                "movetodir" => Self::MoveToDir(MoveToDir::default()),
                _ => Self::None,
            }
        }).collect()
    }
}

impl PipelineAction for PipelineActions {
    fn run(&self, context: &mut PipelineExecutionContext) -> bool {
        match self {
            PipelineActions::MoveToDir(action) => action.run(context),
            PipelineActions::RunCmd(action) => action.run(context),
            PipelineActions::None => false
        }
    }
}

impl Default for PipelineActions {
    fn default() -> Self {
        Self::RunCmd(RunCmd::default())
    }
}