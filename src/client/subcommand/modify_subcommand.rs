use futures::executor::block_on;
use clap::{App, Arg, ArgMatches};

use crate::subcommand::subcommand::SubCommandUtil;
use generated_types::{ModifyHandlerRequest, handler_service_client::HandlerServiceClient};
use super::subcommand::{connect_client, construct_directory_or_all_args, construct_port_arg, construct_server_url, get_path_from_matches_or_current_path};

const STARTUP_TYPES: [&str; 2] = ["auto", "manual"];

#[derive(Clone)]
pub struct ModifySubCommand {}

impl SubCommandUtil for ModifySubCommand {
    fn name(&self) -> &str { "modify" }

    fn alias(&self) -> &str { "mod" }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Modify directory handler")
            .arg(Arg::with_name("debug").short("d")
                .help("print debug information verbosely"))
            .args(construct_directory_or_all_args().as_slice())
            .arg(Arg::with_name("startup").long("startup")
                .help("Set if handler automatically starts on service startup")
                .required(false)
                .takes_value(true)
                .case_insensitive(true)
                .possible_values(&STARTUP_TYPES))
            .arg(Arg::with_name("description").long("description")
                .required(false)
                .takes_value(true))
            .arg(construct_port_arg())
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches) {
        if let Some(server_url) = construct_server_url(sub_matches) {
            match connect_client(server_url) {
                Ok(client) => execute_modify(sub_matches, client),
                Err(e) => println!("{}", e)
            }
        }
        else {

        }
    }
}

fn execute_modify(sub_matches: &ArgMatches, mut client: HandlerServiceClient<tonic::transport::Channel>) {
    let is_auto_startup = match sub_matches.value_of("startup") {
        Some(value) => Some(if value.to_lowercase() == "auto" {true} else {false}),
        None => None
    };
    let modify_description = match sub_matches.value_of("description") {
        Some(description) => Some(description.to_string()),
        None => None
    };
    let mut directory_path = String::new();
    if !sub_matches.is_present("all") {
        let path = get_path_from_matches_or_current_path(sub_matches, "directory").unwrap();
        directory_path = path.into_os_string().into_string().unwrap();
    }
    else {
        match sub_matches.value_of_os("directory") {
            Some(path) => {
                directory_path = path.to_os_string().into_string().unwrap();
            }
            None => {}
        }
    }
    let response = client.modify_handler(ModifyHandlerRequest {
        directory_path,
        is_auto_startup,
        modify_description,
    });
    let response = block_on(response);
    println!("{:?}", response);
}