use serde::{Serialize, Deserialize};

use super::WorkflowAction;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunCmd {
    pub command: String,
}

impl Default for RunCmd {
    fn default() -> Self {
        Self {
            command: String::from("echo $input.file_path"),
        }
    }
}

impl WorkflowAction for RunCmd {
    fn run(&self) {
        todo!()
    }
}