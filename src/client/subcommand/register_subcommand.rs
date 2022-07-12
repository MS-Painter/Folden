use std::path::Path;

use clap::Error as CliError;
use clap::{App, Arg, ArgMatches, ErrorKind};
use futures::executor::block_on;

use super::subcommand_utils::{
    construct_startup_type_arg, get_path_from_matches_or_current_path,
    ExistingDirectoryValueParser, SubCommandUtil,
};
use folden::shared_utils::construct_port_arg;
use generated_types::{handler_service_client::HandlerServiceClient, RegisterToDirectoryRequest};

#[derive(Clone)]
pub struct RegisterSubCommand;

impl SubCommandUtil for RegisterSubCommand {
    fn name(&self) -> &str {
        "register"
    }

    fn alias(&self) -> &str {
        "reg"
    }

    fn requires_connection(&self) -> bool {
        true
    }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Register handler pipeline to directory")
            .arg(
                Arg::with_name("handler_config")
                    .value_name("FILE")
                    .takes_value(true)
                    .required(true)
                    .help("Handler pipeline configuration file"),
            )
            .arg(
                Arg::with_name("directory")
                    .required(false)
                    .empty_values(false)
                    .takes_value(true)
                    .value_parser(ExistingDirectoryValueParser::new())
                    .help("Directory to register to. Leave empty to apply on current"),
            )
            .arg(
                Arg::with_name("start")
                    .long("start")
                    .required(false)
                    .takes_value(false)
                    .help("Start handler on register"),
            )
            .arg(construct_port_arg())
            .arg(construct_startup_type_arg().default_value("manual"))
    }

    fn subcommand_connection_runtime(
        &self,
        sub_matches: &ArgMatches,
        mut client: HandlerServiceClient<tonic::transport::Channel>,
    ) {
        let handler_config_match = sub_matches.value_of("handler_config").unwrap();
        let handler_config_path = Path::new(handler_config_match);
        let is_start_on_register = sub_matches.is_present("start");
        let is_auto_startup = match sub_matches.value_of("startup") {
            Some(value) => value.to_lowercase() == "auto",
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
                match block_on(response) {
                    Ok(response) => println!("{}", response.into_inner().message),
                    Err(e) => println!("{}", e.message()),
                }
            }
            Err(_) => {
                CliError::with_description("Config file doesn't exist".to_string(), ErrorKind::InvalidValue)
                    .exit();
            }
        }
    }
}
