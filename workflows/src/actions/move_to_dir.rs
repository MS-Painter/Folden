use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoveToDir {
    pub directory_name: String,
    pub requires_directory_exists: bool,
    pub replace_older_files: bool,
}

impl Default for MoveToDir {
    fn default() -> Self {
        Self {
            directory_name: String::from("date_file_format"),
            requires_directory_exists: false,
            replace_older_files: true,
        }
    }
}