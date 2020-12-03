use folder_handler::handlers_json::HandlersJson;
use crate::subcommand::subcommand::SubCommandUtil;
use clap::{App, Arg, ArgMatches};
use std::env;

pub struct ThisSubCommand {
    handlers_json: HandlersJson
}

impl SubCommandUtil for ThisSubCommand {
    fn new(handlers_json: HandlersJson) -> Self {
        Self { handlers_json }
    }

    fn name(&self) -> &str {
        "this"
    }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Fun folder usage in current working directory")
            .arg(Arg::with_name("debug").short("d")
                .help("print debug information verbosely"))
            .arg(Arg::with_name("register").short("r").long("register").value_name("FILE").takes_value(true)
                .help("Register new handler type with provided override rust module"))
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches) {
        let path = env::current_dir().unwrap();
        println!("The current directory is {}", path.display());
        println!("{:?}", sub_matches);
        if sub_matches.args.contains_key("register") {
            println!("Desk");
        }
    }
}