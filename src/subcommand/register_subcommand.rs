use std::{env, path::Path};

use futures::executor::block_on;
use tonic::transport::Channel;
use tonic::transport::Error as TransportError;
use clap::Error as CliError;
use clap::{App, Arg, ArgMatches, ErrorKind};

use folder_handler::handlers_json::HandlersJson;
use generated_types::{RegisterToDirectoryRequest, inter_process_client::InterProcessClient};

use crate::subcommand::subcommand::SubCommandUtil;

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
            .arg(RegisterSubCommand::construct_handler_arg("handler", &self.handlers_json))
            .arg(Arg::with_name("handler_config").value_name("FILE")
                .takes_value(true).required(true)
                .help("Handler configuration file"))
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches, client_connect_future: impl futures::Future<Output = Result<InterProcessClient<Channel>, TransportError>>) {
        let handler_config_match = sub_matches.value_of("handler_config").unwrap();
        let handler_config_path = Path::new(handler_config_match);
        if !handler_config_path.exists() {
            CliError::with_description("Config file doesn't exist", ErrorKind::InvalidValue).exit();
        }

        let handler_match = sub_matches.value_of("handler").unwrap();
        let path = env::current_dir().unwrap();

        let mut client = block_on(client_connect_future).unwrap();
        let response = client.register_to_directory(RegisterToDirectoryRequest {
            directory_path: String::from(path.as_os_str().to_str().unwrap()),
            handler_type_name: handler_match.to_string(),
            handler_config_path: handler_config_path.to_str().unwrap().to_string(),
        });

        let response = block_on(response).unwrap().into_inner();
        println!("{:?}", response.message);
    }
}