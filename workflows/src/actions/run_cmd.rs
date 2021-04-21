use serde::{Serialize, Deserialize};

use super::WorkflowAction;
use crate::{workflow_context_input::WorkflowContextInput, workflow_execution_context::WorkflowExecutionContext};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunCmd {
    pub input: WorkflowContextInput,
    pub command: String,
    pub enable_formatting: bool,
}

impl Default for RunCmd {
    fn default() -> Self {
        Self {
            input: WorkflowContextInput::EventFilePath,
            command: String::from("echo $input$"),
            enable_formatting: true,
        }
    }
}

impl WorkflowAction for RunCmd {
    fn run(&self, context: &mut WorkflowExecutionContext) -> bool {
        let formatted_command = if self.enable_formatting {
            Self::format_input(&self.command, context.get_input(self.input)).unwrap_or(self.command.to_owned().into())
        } else {
            self.command.to_owned().into()
        };
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