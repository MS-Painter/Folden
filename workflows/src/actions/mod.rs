use serde::{Serialize, Deserialize};

mod run_cmd;
mod move_to_dir;
use self::{move_to_dir::MoveToDir, run_cmd::RunCmd};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkflowActions {
    MoveToDir(MoveToDir),
    RunCmd(RunCmd),
}

impl WorkflowActions {
    pub fn defaults() -> Vec<WorkflowActions> {
        vec![
            Self::RunCmd(RunCmd::default()),
            Self::MoveToDir(MoveToDir::default()),
        ]
    }
}