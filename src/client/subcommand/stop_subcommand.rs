use std::path::PathBuf;

use futures::executor::block_on;
use clap::{App, Arg, ArgMatches};

use generated_types::{StopHandlerRequest, handler_service_client::HandlerServiceClient};
use crate::subcommand::subcommand::{SubCommandUtil, print_handler_states};
use super::subcommand::{connect_client, construct_directory_or_all_args, construct_port_arg, construct_server_url, get_path_from_matches_or_current_path};

#[derive(Clone)]
pub struct StopSubCommand  {}

impl SubCommandUtil for StopSubCommand {
    fn name(&self) -> &str { "stop" }

    fn alias(&self) -> &str { "" }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Stop handler on directory")
            .arg(Arg::with_name("remove").long("remove")
                .required(false)
                .takes_value(false))
            .args(construct_directory_or_all_args().as_slice())
            .arg(construct_port_arg())
        }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches) {
        if let Some(server_url) = construct_server_url(sub_matches) {
            match connect_client(server_url) {
                Ok(client) => execute_stop(sub_matches, client),
                Err(e) => println!("{}", e)
            }
        }
        else {
            println!("Couldn't send request - No valid endpoint could be parsed");
        }
    }
}

fn execute_stop(sub_matches: &ArgMatches, mut client: HandlerServiceClient<tonic::transport::Channel>) {
    let is_handler_to_be_removed = sub_matches.is_present("remove");
    let mut path = PathBuf::new();
    if !sub_matches.is_present("all") {
        path = get_path_from_matches_or_current_path(sub_matches, "directory").unwrap();
    }
    let response = client.stop_handler(StopHandlerRequest {
        directory_path: String::from(path.as_os_str().to_str().unwrap()),
        remove: is_handler_to_be_removed,
    });
    let response = block_on(response).unwrap().into_inner();
    print_handler_states(response);
}