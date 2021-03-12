use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoveToDir {
    pub directory_name: String,
    pub requires_directory_exists: bool,
    pub replace_older_files: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorkflowActions {
    MoveToDir(MoveToDir)
}