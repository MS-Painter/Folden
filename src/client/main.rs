pub mod subcommand;

use clap::{crate_version, App, AppSettings};

use subcommand::subcommand_utils::{connect_client, construct_server_url, SubCommandCollection};

#[tokio::main]
async fn main() {
    let mut subcommands = SubCommandCollection::new();
    subcommands.add(Box::new(
        subcommand::register_subcommand::RegisterSubCommand {},
    ));
    subcommands.add(Box::new(subcommand::status_subcommand::StatusSubCommand {}));
    subcommands.add(Box::new(subcommand::start_subcommand::StartSubCommand {}));
    subcommands.add(Box::new(subcommand::stop_subcommand::StopSubCommand {}));
    subcommands.add(Box::new(
        subcommand::generate_subcommand::GenerateSubCommand {},
    ));
    subcommands.add(Box::new(subcommand::modify_subcommand::ModifySubCommand {}));
    subcommands.add(Box::new(subcommand::trace_subcommand::TraceSubCommand {}));
    let subcommands_clone = subcommands.clone();

    let app = App::new("Folden")
        .version(crate_version!())
        .about("System-wide folder event handling")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommands(subcommands_clone.collect_as_apps());

    let matches = app.get_matches();
    for subcommand in subcommands {
        if let Some(sub_matches) = subcommand.subcommand_matches(&matches) {
            if subcommand.requires_connection() {
                if let Some(server_url) = construct_server_url(sub_matches) {
                    match connect_client(server_url) {
                        Ok(client) => subcommand.subcommand_connection_runtime(sub_matches, client),
                        Err(e) => println!("{}", e),
                    }
                } else {
                    println!("Couldn't send request - No valid endpoint could be parsed");
                }
            } else {
                subcommand.subcommand_runtime(sub_matches);
            }
            return;
        }
    }
}
