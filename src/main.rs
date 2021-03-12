extern crate clap;
use clap::{App, AppSettings};
use futures::executor::block_on;

use generated_types::inter_process_client::InterProcessClient;
use subcommand::{generate_subcommand::GenerateSubCommand, generate_workflow_subcommand::GenerateWorkflowSubCommand, register_subcommand::RegisterSubCommand, start_subcommand::StartSubCommand, status_subcommand::StatusSubCommand, stop_subcommand::StopSubCommand, subcommand::SubCommandCollection};

mod subcommand;

const GRPC_URL_BASE: &str = "http://localhost:8080/";

#[tokio::main]
async fn main() {
    let mut subcommands = SubCommandCollection::new();
    subcommands.add(Box::new(RegisterSubCommand {}));
    subcommands.add(Box::new(StatusSubCommand {}));
    subcommands.add(Box::new(StartSubCommand {}));
    subcommands.add(Box::new(StopSubCommand {}));
    subcommands.add(Box::new(GenerateSubCommand {}));
    subcommands.add(Box::new(GenerateWorkflowSubCommand {}));
    let subcommands_clone = subcommands.clone();

    let app = App::new("Folden")
    .version("0.1")
    .about("System-wide folder event handling")
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .subcommands(subcommands_clone.collect_as_apps());

    let matches = app.get_matches();
    for subcommand in subcommands {
        if let Some(sub_matches) = subcommand.subcommand_matches(&matches) {
            let client_connect_future = InterProcessClient::connect(GRPC_URL_BASE);
            let client = &mut block_on(client_connect_future).unwrap();
            subcommand.subcommand_runtime(sub_matches, client);
        }
    }
}
