use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PipelineContextInput {
    EventFilePath,
    ActionFilePath,
}
