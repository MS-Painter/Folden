use std::path::PathBuf;

use clap::{App, ArgMatches};
use futures::executor::block_on;

use super::subcommand_utils::{
    construct_directory_or_all_args, construct_simple_output_arg,
    get_path_from_matches_or_current_path, print_handler_states, SubCommandUtil,
};
use folden::shared_utils::construct_port_arg;
use generated_types::{handler_service_client::HandlerServiceClient, StartHandlerRequest};

#[derive(Clone)]
pub struct StartSubCommand;

impl SubCommandUtil for StartSubCommand {
    fn name(&self) -> &str {
        "start"
    }

    fn alias(&self) -> &str {
        ""
    }

    fn requires_connection(&self) -> bool {
        true
    }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Start handler on directory")
            .args(construct_directory_or_all_args().as_slice())
            .arg(construct_port_arg())
            .arg(construct_simple_output_arg())
    }

    fn subcommand_connection_runtime(
        &self,
        sub_matches: &ArgMatches,
        mut client: HandlerServiceClient<tonic::transport::Channel>,
    ) {
        let mut path = PathBuf::new();
        if !sub_matches.is_present("all") {
            path = get_path_from_matches_or_current_path(sub_matches, "directory").unwrap();
        }
        let response = client.start_handler(StartHandlerRequest {
            directory_path: String::from(path.as_os_str().to_str().unwrap()),
        });
        match block_on(response) {
            Ok(response) => print_handler_states(response.into_inner(), sub_matches),
            Err(e) => println!("{}", e.message()),
        }
    }
}
