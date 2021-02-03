use std::error::Error;

extern crate clap;
use clap::{App, AppSettings};

mod subcommand;
use folder_handler::handlers_json::HandlersJson;
use crate::subcommand::subcommand::SubCommandUtil;
use crate::subcommand::start_subcommand::StartSubCommand;
use crate::subcommand::stop_subcommand::StopSubCommand;
use crate::subcommand::status_subcommand::StatusSubCommand;
use crate::subcommand::list_subcommand::ListSubCommand;
use crate::subcommand::generate_subcommand::GenerateSubCommand;
use crate::subcommand::register_subcommand::RegisterSubCommand;

use generated_types::inter_process_client::InterProcessClient;

const GRPC_URL_BASE: &str = "http://localhost:8080/";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client_connect_future = InterProcessClient::connect(GRPC_URL_BASE);

    let handlers_json = HandlersJson::new();
    let register_subcommand = RegisterSubCommand::new(handlers_json.clone());
    let status_subcommand = StatusSubCommand::new(handlers_json.clone());
    let start_subcommand = StartSubCommand::new(handlers_json.clone());
    let stop_subcommand = StopSubCommand::new(handlers_json.clone());
    let list_subcommand = ListSubCommand::new(handlers_json.clone());
    let gen_subcommand = GenerateSubCommand::new(handlers_json.clone());
    let app = App::new("Folden")
        .version("0.1")
        .about("System-wide folder event handling")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(list_subcommand.construct_subcommand())
        .subcommand(start_subcommand.construct_subcommand())
        .subcommand(stop_subcommand.construct_subcommand())
        .subcommand(status_subcommand.construct_subcommand())
        .subcommand(gen_subcommand.construct_subcommand())
        .subcommand(register_subcommand.construct_subcommand());
    let matches = app.get_matches();

    if let Some(sub_matches) = list_subcommand.subcommand_matches(&matches) {
        list_subcommand.subcommand_runtime(sub_matches, client_connect_future);
    } else if let Some(sub_matches) = status_subcommand.subcommand_matches(&matches) {
        status_subcommand.subcommand_runtime(sub_matches, client_connect_future);
    } else if let Some(sub_matches) = start_subcommand.subcommand_matches(&matches) {
        start_subcommand.subcommand_runtime(sub_matches, client_connect_future);
    }else if let Some(sub_matches) = stop_subcommand.subcommand_matches(&matches) {
        stop_subcommand.subcommand_runtime(sub_matches, client_connect_future);
    } else if let Some(sub_matches) = gen_subcommand.subcommand_matches(&matches) {
        gen_subcommand.subcommand_runtime(sub_matches, client_connect_future);
    } else if let Some(sub_matches) = register_subcommand.subcommand_matches(&matches) {
        register_subcommand.subcommand_runtime(sub_matches, client_connect_future);
    }

    Ok(())
}
