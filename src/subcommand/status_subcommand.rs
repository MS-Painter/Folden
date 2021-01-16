use std::env;
use std::error::Error;

use clap::{App, Arg, ArgMatches};

use folder_handler::handlers_json::HandlersJson;
use futures::executor::block_on;
use generated_types::inter_process_client::InterProcessClient;
use tonic::transport::{Channel, Error as TransportError};
use crate::subcommand::subcommand::SubCommandUtil;

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

    fn subcommand_runtime(&self, sub_matches: &ArgMatches, client_connect_future: impl futures::Future<Output = Result<InterProcessClient<Channel>, TransportError>>) {
        let path = env::current_dir().unwrap();
        println!("The current directory is {}", path.display());
        println!("{:?}", sub_matches);
        block_on(client_connect_future).unwrap();
    }
}