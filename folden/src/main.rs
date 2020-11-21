extern crate clap;
use clap::{App, AppSettings};

use crate::subcommand::SubCommandUtil;
mod subcommand;


fn main() {
    let list_subcommand = subcommand::ListSubCommand{};
    let this_subcommand = subcommand::ThisSubCommand{};
    let app = App::new("Folden")
                            .version("0.1")
                            .about("System-wide folder event handling")
                            .setting(AppSettings::SubcommandRequiredElseHelp)
                            .subcommand(list_subcommand.construct_subcommand())
                            .subcommand(this_subcommand.construct_subcommand());
    let matches = app.get_matches();

    if let Some(sub_matches) = list_subcommand.subcommand_matches(&matches) {
        list_subcommand.subcommand_runtime(sub_matches);
    }
    else if let Some(sub_matches) = this_subcommand.subcommand_matches(&matches) {
        this_subcommand.subcommand_runtime(sub_matches);
    }
}
