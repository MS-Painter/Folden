use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PipelineContextInput {
    EventFilePath,
    ActionFilePath,
}