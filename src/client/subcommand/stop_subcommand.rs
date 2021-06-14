use std::path::PathBuf;

use clap::{App, Arg, ArgMatches};
use futures::executor::block_on;

use super::subcommand_utils::{
    construct_directory_or_all_args, construct_simple_output_arg,
    get_path_from_matches_or_current_path, print_handler_states, SubCommandUtil,
};
use folden::shared_utils::construct_port_arg;
use generated_types::{handler_service_client::HandlerServiceClient, StopHandlerRequest};

#[derive(Clone)]
pub struct StopSubCommand;

impl SubCommandUtil for StopSubCommand {
    fn name(&self) -> &str {
        "stop"
    }

    fn alias(&self) -> &str {
        ""
    }

    fn requires_connection(&self) -> bool {
        true
    }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Stop handler on directory")
            .arg(
                Arg::with_name("remove")
                    .long("remove")
                    .visible_alias("rm")
                    .required(false)
                    .takes_value(false)
                    .help("Deregister handler from directory"),
            )
            .arg(construct_port_arg())
            .arg(construct_simple_output_arg())
            .args(construct_directory_or_all_args().as_slice())
    }

    fn subcommand_connection_runtime(
        &self,
        sub_matches: &ArgMatches,
        mut client: HandlerServiceClient<tonic::transport::Channel>,
    ) {
        let is_handler_to_be_removed = sub_matches.is_present("remove");
        let mut path = PathBuf::new();
        if !sub_matches.is_present("all") {
            path = get_path_from_matches_or_current_path(sub_matches, "directory").unwrap();
        }
        let response = client.stop_handler(StopHandlerRequest {
            directory_path: String::from(path.as_os_str().to_str().unwrap()),
            remove: is_handler_to_be_removed,
        });
        match block_on(response) {
            Ok(response) => print_handler_states(response.into_inner(), sub_matches),
            Err(e) => println!("{}", e.message()),
        }
    }
}
