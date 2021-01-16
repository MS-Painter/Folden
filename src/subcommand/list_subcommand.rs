use folder_handler::handlers_json::HandlersJson;
use crate::subcommand::subcommand::SubCommandUtil;
use clap::{Arg, App, ArgMatches};

pub struct ListSubCommand {
    handlers_json: HandlersJson
}

impl SubCommandUtil for ListSubCommand {
    fn new(handlers_json: HandlersJson) -> Self {
        Self { handlers_json }
    }

    fn name(&self) -> &str {
        "list"
    }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Details on usage of fun folder across entire filesystem")
            .arg(Arg::with_name("debug").short("d")
                .help("print debug information verbosely"))
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches, client_connect_future: impl futures::Future<Output = Result<generated_types::inter_process_client::InterProcessClient<tonic::transport::Channel>, tonic::transport::Error>>) {
        println!("{:?}", &sub_matches.args.len())
    }
}