use folder_handler::handlers_json::HandlersJson;
use crate::subcommand::subcommand::SubCommandUtil;
use clap::{App, Arg, ArgMatches};
use std::env;

pub struct RegisterSubCommand {
    handlers_json: HandlersJson
}

impl SubCommandUtil for RegisterSubCommand {
    fn new(handlers_json: HandlersJson) -> Self {
        Self { handlers_json }
    }

    fn name(&self) -> &str {
        "register"
    }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Register handler to directory")
            .arg(Arg::with_name("debug").short("d")
                .help("print debug information verbosely"))
            .arg(Arg::with_name("handler_config").value_name("FILE")
                .takes_value(true).required(true)
                .help("Handler configuration file"))
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches, client_connect_future: impl futures::Future<Output = Result<generated_types::inter_process_client::InterProcessClient<tonic::transport::Channel>, tonic::transport::Error>>) {
        let handler_config_match = sub_matches.value_of("handler_config").unwrap();
        let path = env::current_dir().unwrap();
        println!("The current directory is {}", path.display());
        println!("{:?}", sub_matches);
    }
}