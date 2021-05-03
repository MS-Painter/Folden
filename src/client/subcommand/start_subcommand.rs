use std::path::PathBuf;

use clap::{App, ArgMatches};
use futures::executor::block_on;

use crate::subcommand::subcommand::SubCommandUtil;
use generated_types::{StartHandlerRequest, handler_service_client::HandlerServiceClient};
use super::subcommand::{connect_client, construct_directory_or_all_args, construct_simple_output_arg, construct_port_arg, construct_server_url, get_path_from_matches_or_current_path, print_handler_states};

#[derive(Clone)]
pub struct StartSubCommand  {}

impl SubCommandUtil for StartSubCommand {
    fn name(&self) -> &str { "start" }

    fn alias(&self) -> &str { "" }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Start handler on directory")
            .args(construct_directory_or_all_args().as_slice())
            .arg(construct_port_arg())
            .arg(construct_simple_output_arg())
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches) {
        if let Some(server_url) = construct_server_url(sub_matches) {
            match connect_client(server_url) {
                Ok(client) => execute_start(sub_matches, client),
                Err(e) => println!("{}", e)
            }
        }
        else {
            println!("Couldn't send request - No valid endpoint could be parsed");
        }
    }
}

fn execute_start(sub_matches: &ArgMatches, mut client: HandlerServiceClient<tonic::transport::Channel>) {
    let mut path = PathBuf::new();
    if !sub_matches.is_present("all") {
        path = get_path_from_matches_or_current_path(sub_matches, "directory").unwrap();
    }
    let response = client.start_handler(StartHandlerRequest {
        directory_path: String::from(path.as_os_str().to_str().unwrap()),
    });
    let response = block_on(response).unwrap().into_inner();
    print_handler_states(response, sub_matches);
}