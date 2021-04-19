use tonic::transport::Channel;
use futures::executor::block_on;
use clap::{App, Arg, ArgMatches};

use crate::subcommand::subcommand::SubCommandUtil;
use generated_types::{GetDirectoryStatusRequest, handler_service_client::HandlerServiceClient};
use super::subcommand::{construct_directory_or_all_args, get_path_from_matches_or_current_path};

#[derive(Clone)]
pub struct StatusSubCommand {}

impl SubCommandUtil for StatusSubCommand {
    fn name(&self) -> &str { "status" }

    fn alias(&self) -> &str { "stat" }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Fun folder usage in current working directory")
            .arg(Arg::with_name("debug").short("d")
                .help("print debug information verbosely"))
            .args(construct_directory_or_all_args().as_slice())
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches, client: &mut HandlerServiceClient<Channel>) {  
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
        let response = client.get_directory_status(GetDirectoryStatusRequest {
            directory_path
        });
        let response = block_on(response).unwrap().into_inner();
        println!("{:?}", response.directory_states_map);
    }
}