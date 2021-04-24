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

pub trait WorkflowAction {
    // Execute action. Returns if action deemed successful.
    fn run(&self, context: &mut PipelineExecutionContext) -> bool;

    fn format_input(text: &String, input: Option<PathBuf>) -> Result<Cow<str>,()> {
        if let Some(input) = input {
            lazy_static! {
                static ref INPUT_RE: Regex = Regex::new(r"(\$input\$)").unwrap();
            }
            return Ok(INPUT_RE.replace_all(text, input.to_string_lossy()))
        }
        Err(())
    }

    fn format_datetime<S>(text: S) -> String where S: AsRef<str> {
        chrono::Local::now().format(text.as_ref()).to_string()
    }
}

pub fn construct_working_dir(input_path: &PathBuf, directory_path: &PathBuf) -> PathBuf {
    let mut working_path = PathBuf::from(input_path.parent().unwrap());
    working_path.push(directory_path); // If directory_path is absolute will replace the entire path
    working_path
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkflowActions {
    MoveToDir(MoveToDir),
    RunCmd(RunCmd),
    None,
}

impl WorkflowActions {
    pub fn defaults<'a, I>(actions: I) -> Vec<WorkflowActions> 
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

impl WorkflowAction for WorkflowActions {
    fn run(&self, context: &mut PipelineExecutionContext) -> bool {
        match self {
            WorkflowActions::MoveToDir(action) => action.run(context),
            WorkflowActions::RunCmd(action) => action.run(context),
            WorkflowActions::None => false
        }
    }
}

impl Default for WorkflowActions {
    fn default() -> Self {
        Self::RunCmd(RunCmd::default())
    }
}