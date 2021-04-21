use serde::{Serialize, Deserialize};

use super::WorkflowAction;
use crate::{workflow_context_input::WorkflowContextInput, workflow_execution_context::WorkflowExecutionContext};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunCmd {
    pub input: WorkflowContextInput,
    pub command: String,
    pub input_formatting: bool,
    pub datetime_formatting: bool,
}

impl RunCmd {
    fn format_command(&self, context: &mut WorkflowExecutionContext) -> std::borrow::Cow<str> {
        let mut formatted_command = self.command.to_owned().into();
        if self.input_formatting {
            formatted_command = Self::format_input(&self.command, context.get_input(self.input))
            .unwrap_or(self.command.to_owned().into());
        } 
        if self.datetime_formatting {
            formatted_command = Self::format_datetime(formatted_command).into();
        };
        formatted_command
    }
}

impl Default for RunCmd {
    fn default() -> Self {
        Self {
            input: WorkflowContextInput::EventFilePath,
            command: String::from("echo $input$"),
            input_formatting: true,
            datetime_formatting: true,
        }
    }
}

impl WorkflowAction for RunCmd {
    fn run(&self, context: &mut WorkflowExecutionContext) -> bool {
        let formatted_command = self.format_command(context);
        match Self::spawn_command(&formatted_command, context) {
            Ok(process) => {
                let output = process.wait_with_output();
                return match output {
                    Ok(out) => {
                        if out.stdout.is_empty() {
                            let stderr = String::from_utf8(out.stderr).unwrap();
                            println!("RunCmd stderr - {:?}", stderr);
                            false
                        }
                        else {
                            let stdout = String::from_utf8(out.stdout).unwrap();
                            println!("RunCmd stdout - {:?}", stdout);
                            true
                        }
                    }
                    Err(e) => {
                        println!("RunCmd stderr - {:?}", e);
                        false
                    }
                }
            }
            Err(e) => {
                println!("RunCmd could not spawn command.\nCommand: {:?}\nError: {:?}", formatted_command, e);
                false
            }
        }
    }
}