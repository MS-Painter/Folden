use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum WorkflowContextInput {
    EventFilePath,
    ActionFilePath,
}