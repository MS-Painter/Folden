use std::path::PathBuf;

use regex::Regex;
use serde::{Serialize, Deserialize};

mod run_cmd;
mod move_to_dir;
use self::{move_to_dir::MoveToDir, run_cmd::RunCmd};
use crate::workflow_execution_context::WorkflowExecutionContext;

pub const ACTION_TYPES: [&str; 2] = ["runcmd", "movetodir"];

pub trait WorkflowAction {
    // Execute action. Returns if action deemed successful.
    fn run(&self, context: &mut WorkflowExecutionContext) -> bool;

    fn format_input(text: &String, input: Option<PathBuf>) -> Result<String,()> {
        if let Some(input) = input {
            let re = Regex::new(r#"(\$input\$)"#).unwrap();
            return Ok(re.replace(text, input.to_str().unwrap()).to_string());
        }
        Err(())
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
    fn run(&self, context: &mut WorkflowExecutionContext) -> bool {
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