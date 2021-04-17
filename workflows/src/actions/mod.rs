use std::path::PathBuf;

use serde::{Serialize, Deserialize};

mod run_cmd;
mod move_to_dir;
use self::{move_to_dir::MoveToDir, run_cmd::RunCmd};
use crate::workflow_execution_context::WorkflowExecutionContext;

pub trait WorkflowAction {
    // Execute action. Returns if action deemed successful.
    fn run(&self, context: &mut WorkflowExecutionContext) -> bool;
    fn construct_working_dir(&self, input_path: &PathBuf) -> PathBuf;
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

    fn construct_working_dir(&self, input_path: &PathBuf) -> PathBuf {
        match self {
            WorkflowActions::MoveToDir(action) => action.construct_working_dir(input_path),
            WorkflowActions::RunCmd(action) => action.construct_working_dir(input_path),
            WorkflowActions::None => PathBuf::new()
        }
    }
}