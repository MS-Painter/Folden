use clap::{App, AppSettings};

use subcommand::subcommand::SubCommandCollection;

mod subcommand;

#[tokio::main]
async fn main() {
    let mut subcommands = SubCommandCollection::new();
    subcommands.add(Box::new(subcommand::register_subcommand::RegisterSubCommand {}));
    subcommands.add(Box::new(subcommand::status_subcommand::StatusSubCommand {}));
    subcommands.add(Box::new(subcommand::start_subcommand::StartSubCommand {}));
    subcommands.add(Box::new(subcommand::stop_subcommand::StopSubCommand {}));
    subcommands.add(Box::new(subcommand::generate_subcommand::GenerateSubCommand {}));
    subcommands.add(Box::new(subcommand::modify_subcommand::ModifySubCommand {}));
    let subcommands_clone = subcommands.clone();

    let app = App::new("Folden")
        .version("0.1")
        .about("System-wide folder event handling")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommands(subcommands_clone.collect_as_apps());

    let matches = app.get_matches();
    for subcommand in subcommands {
        if let Some(sub_matches) = subcommand.subcommand_matches(&matches) {
            subcommand.subcommand_runtime(sub_matches);
        }
    }
}
