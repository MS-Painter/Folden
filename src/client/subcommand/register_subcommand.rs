use std::path::Path;

use clap::Error as CliError;
use futures::executor::block_on;
use clap::{App, Arg, ArgMatches, ErrorKind};

use crate::subcommand::subcommand::SubCommandUtil;
use generated_types::{RegisterToDirectoryRequest, handler_service_client::HandlerServiceClient};
use super::subcommand::{connect_client, construct_port_arg, construct_server_url, construct_startup_type_arg, get_path_from_matches_or_current_path, is_existing_directory_validator};

#[derive(Clone)]
pub struct RegisterSubCommand {}

impl SubCommandUtil for RegisterSubCommand {
    fn name(&self) -> &str { "register" }

    fn alias(&self) -> &str { "reg" }
 
    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Register handler pipeline to directory")
            .arg(Arg::with_name("handler_config").value_name("FILE")
                .takes_value(true).required(true)
                .help("Handler pipeline configuration file"))
            .arg(Arg::with_name("directory")
                .required(false)
                .empty_values(false)
                .takes_value(true)
                .validator_os(is_existing_directory_validator)
                .help("Directory to register to. Leave empty to apply on current"))
            .arg(Arg::with_name("start").long("start")
                .required(false)
                .takes_value(false)
                .help("Start handler on register"))
            .arg(construct_port_arg())
            .arg(construct_startup_type_arg().default_value("manual"))
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches) {
        if let Some(server_url) = construct_server_url(sub_matches) {
            match connect_client(server_url) {
                Ok(client) => execute_register(sub_matches, client),
                Err(e) => println!("{}", e)
            }
        }
        else {
            println!("Couldn't send request - No valid endpoint could be parsed");
        }
    }
}

fn execute_register(sub_matches: &ArgMatches, mut client: HandlerServiceClient<tonic::transport::Channel>) {
    let handler_config_match = sub_matches.value_of("handler_config").unwrap();
    let handler_config_path = Path::new(handler_config_match);
    let is_start_on_register = sub_matches.is_present("start");
    let is_auto_startup = match sub_matches.value_of("startup") {
        Some(value) => if value.to_lowercase() == "auto" {true} else {false},
        None => false,
    };
    match handler_config_path.canonicalize() {
        Ok(handler_config_abs_path) => {
            let path = get_path_from_matches_or_current_path(sub_matches, "directory").unwrap();
            let response = client.register_to_directory(RegisterToDirectoryRequest {
                directory_path: String::from(path.as_os_str().to_str().unwrap()),
                handler_config_path: handler_config_abs_path.to_str().unwrap().to_string(),
                is_start_on_register,
                is_auto_startup,
            });
            let response = block_on(response).unwrap().into_inner();
            println!("{:?}", response.message);
        }
        Err(_) => {
            CliError::with_description("Config file doesn't exist", ErrorKind::InvalidValue).exit();
        }
    }
}