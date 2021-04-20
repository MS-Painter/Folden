use std::path::PathBuf;

use tonic::transport::Channel;
use futures::executor::block_on;
use clap::{App, Arg, ArgMatches};

use crate::subcommand::subcommand::SubCommandUtil;
use generated_types::{StopHandlerRequest, handler_service_client::HandlerServiceClient};
use super::subcommand::{construct_directory_or_all_args, get_path_from_matches_or_current_path};

#[derive(Clone)]
pub struct StopSubCommand  {}

impl SubCommandUtil for StopSubCommand {
    fn name(&self) -> &str { "stop" }

    fn alias(&self) -> &str { "" }

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

    fn subcommand_runtime(&self, sub_matches: &ArgMatches, client: &mut HandlerServiceClient<Channel>) {
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
        println!("{:?}", response.states_map);
    }
}