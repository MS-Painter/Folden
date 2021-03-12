use serde::{Serialize, Deserialize};

mod run_cmd;
mod move_to_dir;
use self::{move_to_dir::MoveToDir, run_cmd::RunCmd};

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
            match action_name {
                "runcmd" => Self::RunCmd(RunCmd::default()),
                "movetodir" => Self::MoveToDir(MoveToDir::default()),
                _ => Self::None,
            }
        }).collect()
    }
}