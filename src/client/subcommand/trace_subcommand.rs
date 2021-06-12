use clap::{App, ArgMatches};
use futures::executor::block_on;

use crate::subcommand::subcommand::SubCommandUtil;
use generated_types::{GetDirectoryStatusRequest, TraceHandlerRequest, handler_service_client::HandlerServiceClient};
use super::subcommand::{construct_directory_or_all_args, construct_port_arg, get_path_from_matches_or_current_path};

#[derive(Clone)]
pub struct TraceSubCommand {}

impl SubCommandUtil for TraceSubCommand {
    fn name(&self) -> &str { "trace" }

    fn alias(&self) -> &str { "" }

    fn requires_connection(&self) -> bool { true }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Trace directory handler output")
            .arg(construct_port_arg())
            .args(construct_directory_or_all_args().as_slice())
    }

    fn subcommand_connection_runtime(&self, sub_matches: &ArgMatches, mut client: HandlerServiceClient<tonic::transport::Channel>) {
        let mut directory_path = String::new();
        let trace_all_directories = sub_matches.is_present("all");
        if !trace_all_directories {
            let path = get_path_from_matches_or_current_path(sub_matches, "directory").unwrap();
            directory_path = path.into_os_string().into_string().unwrap();
        }
        let response = client.trace_handler(TraceHandlerRequest {
            directory_path: directory_path.clone(),
        });
        let response = block_on(response);
        match response {
            Ok(response) => {
                let mut stream = response.into_inner();
                loop {
                    match block_on(stream.message()) {
                        Ok(response) => {
                            let response = response.unwrap();
                            if trace_all_directories {
                                print_response(response, true);
                            }
                            else if response.directory_path == directory_path {
                                print_response(response, false);
                            }
                        },
                        Err(status) => {
                            println!("{}", status);
                            match status.code() {
                                tonic::Code::Cancelled => {
                                    if trace_all_directories {
                                        let response = client.get_directory_status(GetDirectoryStatusRequest {
                                            directory_path: String::new()
                                        });
                                        if let Ok(response) = block_on(response) {
                                            let response = response.into_inner();
                                            if response.summary_map.is_empty() || 
                                            response.summary_map.iter().all(|(_dir, handler)| !handler.is_alive) {
                                                break;
                                            }
                                        }
                                    }
                                    else {
                                        break;
                                    }
                                },
                                _ => break,
                            }
                        },
                    }
                }
            }
            Err(e) => println!("{}", e)
        }
    }
}

fn print_response(response: generated_types::TraceHandlerResponse, print_directory: bool) {
    if print_directory {
        println!("
        Directory - {}
        Action - {}
        Message - {}
        ", response.directory_path, response.action.unwrap_or("None".to_string()), response.message);
    }
    else {
        println!("
        Action - {}
        Message - {}
        ", response.action.unwrap_or("None".to_string()), response.message);
    }
}