use serde::{Serialize, Deserialize};

mod run_cmd;
mod move_to_dir;
use self::{move_to_dir::MoveToDir, run_cmd::RunCmd};
use crate::workflow_execution_context::WorkflowExecutionContext;

pub trait WorkflowAction {
    fn run(&self, context: &mut WorkflowExecutionContext);
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
    fn run(&self, context: &mut WorkflowExecutionContext) {
        match self {
            WorkflowActions::MoveToDir(action) => action.run(context),
            WorkflowActions::RunCmd(action) => action.run(context),
            WorkflowActions::None => {}
        }
    }
}