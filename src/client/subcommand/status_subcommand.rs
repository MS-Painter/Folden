use futures::executor::block_on;
use clap::{App, Arg, ArgMatches};

use crate::subcommand::subcommand::SubCommandUtil;
use generated_types::{GetDirectoryStatusRequest, handler_service_client::HandlerServiceClient};
use super::subcommand::{connect_client, construct_directory_or_all_args, construct_port_arg, construct_server_url, get_path_from_matches_or_current_path};

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
            .arg(construct_port_arg())
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches) {
        if let Some(server_url) = construct_server_url(sub_matches) {
            match connect_client(server_url) {
                Ok(client) => execute_status(sub_matches, client),
                Err(_) => {}
            }   
        }
        else {

        }
    }
}

fn execute_status(sub_matches: &ArgMatches, mut client: HandlerServiceClient<tonic::transport::Channel>) {
    let mut directory_path = String::new();
    let all_directories = sub_matches.is_present("all");
    if all_directories {
        match sub_matches.value_of_os("directory") {
            Some(path) => {
                directory_path = path.to_os_string().into_string().unwrap();
            }
            None => {}
        }
    }
    else {
        let path = get_path_from_matches_or_current_path(sub_matches, "directory").unwrap();
        directory_path = path.into_os_string().into_string().unwrap();
    }
    let response = client.get_directory_status(GetDirectoryStatusRequest {
        directory_path
    });
    let response = block_on(response).unwrap().into_inner();
    if response.directory_states_map.is_empty() {
        println!("No handler registered on {}", if all_directories {"file system"} else {"directory"});
    }
    else {
        println!("{:?}", response.directory_states_map);
    }
}