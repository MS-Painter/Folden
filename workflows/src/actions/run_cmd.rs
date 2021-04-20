use std::process::{Command, Stdio};

use serde::{Serialize, Deserialize};

use super::WorkflowAction;
use crate::{workflow_context_input::WorkflowContextInput, workflow_execution_context::WorkflowExecutionContext};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunCmd {
    pub input: WorkflowContextInput,
    pub command: String,
}

impl Default for RunCmd {
    fn default() -> Self {
        Self {
            input: WorkflowContextInput::EventFilePath,
            command: String::from("echo $input$"),
        }
    }
}

impl WorkflowAction for RunCmd {
    fn run(&self, context: &mut WorkflowExecutionContext) -> bool {
        let formatted_command = Self::format_input(&self.command, context.get_input(self.input)).unwrap_or(self.command.to_owned());
        let mut command = Command::new(&formatted_command);
        command.stdout(Stdio::piped());
        command.current_dir(&context.event_file_path.parent().unwrap());
        match command.spawn() {
            Ok(process) => {
                let output = process.wait_with_output();
                match &output {
                    Ok(out) => println!("RunCmd stdout - {:?}", out.stdout.as_slice()),
                    Err(e) => println!("RunCmd stderr - {:?}", e)
                }
                output.is_ok()
            }
            Err(e) => {
                println!("RunCmd error - Command could not spawn.\nError: {:?}", e);
                false
            }
        }
    }
}