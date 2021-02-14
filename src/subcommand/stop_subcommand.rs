use std::path::PathBuf;

use futures::executor::block_on;
use clap::{App, Arg, ArgMatches};
use tonic::transport::{Channel, Error as TransportError};

use crate::subcommand::subcommand::SubCommandUtil;
use super::subcommand::{construct_directory_or_all_args, get_path_from_matches_or_current_path};
use generated_types::StopHandlerRequest;
use generated_types::inter_process_client::InterProcessClient;


pub struct StopSubCommand  {}

impl SubCommandUtil for StopSubCommand {
    fn name(&self) -> &str { "stop" }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Stop handler on directory")
            .arg(Arg::with_name("debug").short("d")
                .help("print debug information verbosely"))
            .arg(Arg::with_name("remove").long("remove")
                .required(false)
                .takes_value(false))
            .args(construct_directory_or_all_args().as_slice())
        }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches, client_connect_future: impl futures::Future<Output = Result<InterProcessClient<Channel>, TransportError>>) {
        let is_handler_to_be_removed = sub_matches.is_present("remove");
        let mut path = PathBuf::new();
        if !sub_matches.is_present("all") {
            path = get_path_from_matches_or_current_path(sub_matches, "directory").unwrap();
        }
        
        let mut client = block_on(client_connect_future).unwrap();
        let response = client.stop_handler(StopHandlerRequest {
            directory_path: String::from(path.as_os_str().to_str().unwrap()),
            remove: is_handler_to_be_removed,
        });
        let response = block_on(response).unwrap().into_inner();
        println!("{:?}", response.states_map);
    }
}