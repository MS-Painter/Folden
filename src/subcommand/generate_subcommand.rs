use folder_handler::handlers_json::HandlersJson;
use clap::{App, Arg, ArgMatches};
use crate::subcommand::subcommand::SubCommandUtil;

pub struct GenerateSubCommand {
    handlers_json: HandlersJson
}

impl SubCommandUtil for GenerateSubCommand {
    fn new(handlers_json: HandlersJson) -> Self {
        Self { handlers_json }
    }

    fn name(&self) -> &str { "generate" }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Generate default handler config for input registered handler")
            .arg(Arg::with_name("debug")
                .short("d")
                .help("print debug information verbosely"))
            .arg(Arg::with_name("handler")
                .required(true)
                .empty_values(false)
                .case_insensitive(true)
                .possible_values(self.handlers_json.get_handler_types().as_slice()))
            .arg(Arg::with_name("path")
                .required(false))
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches) {
        let handler_match = sub_matches.value_of("handler").unwrap();
        match self.handlers_json.get_handler_by_name(&handler_match) {
            Ok(_) => println!("YO"),
            Err(e) => panic!(e)
        }
    }
}
