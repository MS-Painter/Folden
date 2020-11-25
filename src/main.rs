extern crate clap;
use clap::{App, AppSettings};

use crate::subcommand::SubCommandUtil;
use folder_handler::handlers_json::HandlersJson;

mod subcommand;

fn main() {
    let handlers_json = HandlersJson::new();
    let list_subcommand = subcommand::ListSubCommand::new(handlers_json.clone());
    let this_subcommand = subcommand::ThisSubCommand::new(handlers_json.clone());
    let gen_subcommand = subcommand::GenerateSubCommand::new(handlers_json.clone());
    let app = App::new("Folden")
                            .version("0.1")
                            .about("System-wide folder event handling")
                            .setting(AppSettings::SubcommandRequiredElseHelp)
                            .subcommand(list_subcommand.construct_subcommand())
                            .subcommand(this_subcommand.construct_subcommand())
                            .subcommand(gen_subcommand.construct_subcommand());
    let matches = app.get_matches();

    if let Some(sub_matches) = list_subcommand.subcommand_matches(&matches) {
        list_subcommand.subcommand_runtime(sub_matches);
    }
    else if let Some(sub_matches) = this_subcommand.subcommand_matches(&matches) {
        this_subcommand.subcommand_runtime(sub_matches);
    }
    else if let Some(sub_matches) = gen_subcommand.subcommand_matches(&matches) {
        gen_subcommand.subcommand_runtime(sub_matches);
    }
}
