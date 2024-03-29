use std::process::{Child, Command, Stdio};

use serde::{Deserialize, Serialize};

use super::PipelineAction;
use crate::{
    pipeline_context_input::PipelineContextInput,
    pipeline_execution_context::PipelineExecutionContext,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunCmd {
    pub input: PipelineContextInput,
    pub command: String,
    pub input_formatting: bool,
    pub datetime_formatting: bool,
}

impl RunCmd {
    fn format_command(&self, context: &mut PipelineExecutionContext) -> String {
        let mut formatted_command = self.command.to_owned();
        if self.input_formatting {
            if let Some(input_path) = context.get_input(self.input) {
                formatted_command = Self::format_input(&self.command, input_path).to_string();
            }
        }
        if self.datetime_formatting {
            formatted_command = Self::format_datetime(formatted_command);
        };
        formatted_command
    }
}

impl PipelineAction for RunCmd {
    fn run(&self, context: &mut PipelineExecutionContext) -> bool {
        let formatted_command = self.format_command(context);
        match spawn_command(&formatted_command, context) {
            Ok(process) => {
                let output = process.wait_with_output();
                return match output {
                    Ok(out) => {
                        if out.stdout.is_empty() {
                            let stderr = String::from_utf8(out.stderr).unwrap();
                            context.handle_error(format!("Stderr - {:?}", stderr))
                        } else {
                            let stdout = String::from_utf8(out.stdout).unwrap();
                            context.log(format!("Stdout - {:?}", stdout));
                            true
                        }
                    }
                    Err(e) => context.handle_error(format!("Error - {:?}", e)),
                };
            }
            Err(e) => context.handle_error(format!(
                "Could not spawn command.\nCommand: {:?}\nError: {:?}",
                formatted_command, e
            )),
        }
    }
}

impl Default for RunCmd {
    fn default() -> Self {
        Self {
            input: PipelineContextInput::EventFilePath,
            command: String::from("echo $input$"),
            input_formatting: true,
            datetime_formatting: true,
        }
    }
}

fn spawn_command<S>(input: &S, context: &mut PipelineExecutionContext) -> std::io::Result<Child>
where
    S: AsRef<str>,
{
    let parent_dir_path = context.event_file_path.parent().unwrap();
    if cfg!(windows) {
        Command::new("cmd.exe")
            .arg(format!("/C {}", input.as_ref()))
            .current_dir(parent_dir_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else {
        Command::new(input.as_ref())
            .current_dir(parent_dir_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}
