use folder_handler::handlers_json::HandlersJson;
use crate::subcommand::subcommand::SubCommandUtil;
use clap::{App, Arg, ArgMatches};
use std::env;

pub struct StatusSubCommand {
    handlers_json: HandlersJson
}

impl SubCommandUtil for StatusSubCommand {
    fn new(handlers_json: HandlersJson) -> Self {
        Self { handlers_json }
    }

    fn name(&self) -> &str {
        "status"
    }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Fun folder usage in current working directory")
            .arg(Arg::with_name("debug").short("d")
                .help("print debug information verbosely"))
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches) {
        let path = env::current_dir().unwrap();
        println!("The current directory is {}", path.display());
        println!("{:?}", sub_matches);
    }
}