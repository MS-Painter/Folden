use std::string::ToString;

use serde::{Serialize, Deserialize};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, ToString};

mod run_cmd;
mod move_to_dir;
use crate::workflow_execution_context::WorkflowExecutionContext;
use self::{move_to_dir::MoveToDir, run_cmd::RunCmd};

pub trait WorkflowAction {
    fn run(&self, context: &mut WorkflowExecutionContext);
}

#[derive(Clone, Debug, EnumIter, ToString, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkflowActions {
    MoveToDir(MoveToDir),
    RunCmd(RunCmd),
    #[strum(disabled)]
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

    
    pub fn names() -> Vec<String> {
        WorkflowActions::iter().map(|action| action.to_string()).collect()
    }
}

impl WorkflowAction for WorkflowActions {
    fn run(&self, context: &mut WorkflowExecutionContext) {
        match self {
            WorkflowActions::MoveToDir(action) => action.run(context),
            WorkflowActions::RunCmd(action) => action.run(context),
            WorkflowActions::None => {}
        }
    }
}