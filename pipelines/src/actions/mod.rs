use std::{borrow::Cow, path::{Path, PathBuf}};

use regex::Regex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumVariantNames, IntoStaticStr};

mod run_cmd;
mod move_to_dir;
use self::{move_to_dir::MoveToDir, run_cmd::RunCmd};
use crate::pipeline_execution_context::PipelineExecutionContext;

pub trait PipelineAction {
    // Execute action. Returns if action deemed successful.
    fn run(&self, context: &mut PipelineExecutionContext) -> bool;

    fn format_input(text: &str, input: PathBuf) -> Cow<str> {
        lazy_static! {
            static ref INPUT_RE: Regex = Regex::new(r"(\$input\$)").unwrap();
        }
        INPUT_RE.replace_all(text, input.to_string_lossy())
    }

    fn format_datetime<S>(text: S) -> String where S: AsRef<str> {
        chrono::Local::now().format(text.as_ref()).to_string()
    }
}

pub fn construct_working_dir(input_path: &Path, directory_path: &Path) -> PathBuf {
    let mut working_path = PathBuf::from(input_path.parent().unwrap());
    working_path.push(directory_path); // If directory_path is absolute will replace the entire path
    working_path
}

#[derive(Clone, Debug, EnumVariantNames, IntoStaticStr, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PipelineActions {
    MoveToDir(MoveToDir),
    RunCmd(RunCmd),
}

impl PipelineActions {
    pub fn defaults<'a, I>(actions: I) -> Vec<PipelineActions> 
    where I: Iterator<Item = &'a str> {
        actions.map(|action_name| {
            match action_name.to_lowercase().as_str() {
                "runcmd" => Self::RunCmd(RunCmd::default()),
                "movetodir" => Self::MoveToDir(MoveToDir::default()),
                _ => panic!("Incompatible action provided"),
            }
        }).collect()
    }
}

impl PipelineAction for PipelineActions {
    fn run(&self, context: &mut PipelineExecutionContext) -> bool {
        match self {
            PipelineActions::MoveToDir(action) => action.run(context),
            PipelineActions::RunCmd(action) => action.run(context)
        }
    }
}

impl Default for PipelineActions {
    fn default() -> Self {
        Self::RunCmd(RunCmd::default())
    }
}