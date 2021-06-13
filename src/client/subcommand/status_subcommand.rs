use clap::{App, ArgMatches};
use futures::executor::block_on;

use folden::shared_utils::construct_port_arg;
use generated_types::{GetDirectoryStatusRequest, handler_service_client::HandlerServiceClient};
use super::subcommand_utils::{SubCommandUtil, construct_directory_or_all_args, construct_simple_output_arg, get_path_from_matches_or_current_path, print_handler_summaries};

#[derive(Clone)]
pub struct StatusSubCommand {}

impl SubCommandUtil for StatusSubCommand {
    fn name(&self) -> &str { "status" }

    fn alias(&self) -> &str { "stat" }

    fn requires_connection(&self) -> bool { true }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Status of a registered handler given a directory")
            .args(construct_directory_or_all_args().as_slice())
            .arg(construct_port_arg())
            .arg(construct_simple_output_arg())
    }

    fn subcommand_connection_runtime(&self, sub_matches: &ArgMatches, mut client: HandlerServiceClient<tonic::transport::Channel>) {
        let mut directory_path = String::new();
        let all_directories = sub_matches.is_present("all");
        if all_directories {
            if let Some(path) = sub_matches.value_of_os("directory") {
                directory_path = path.to_os_string().into_string().unwrap();
            }
        }
        else {
            let path = get_path_from_matches_or_current_path(sub_matches, "directory").unwrap();
            directory_path = path.into_os_string().into_string().unwrap();
        }
        let response = client.get_directory_status(GetDirectoryStatusRequest {
            directory_path
        });
        match block_on(response) {
            Ok(response) => {
                let response = response.into_inner();
                if response.summary_map.is_empty() {
                    println!("No handler registered on {}", if all_directories {"file system"} else {"directory"});
                }
                else {
                    print_handler_summaries(response, sub_matches);
                }
            }
            Err(e) => println!("{}", e.message())
        }
    }
}