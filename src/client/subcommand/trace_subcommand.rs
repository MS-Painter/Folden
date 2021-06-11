use clap::{App, ArgMatches};
use futures::executor::block_on;

use crate::subcommand::subcommand::SubCommandUtil;
use generated_types::{TraceHandlerRequest, handler_service_client::HandlerServiceClient};
use super::subcommand::{connect_client, construct_directory_or_all_args, construct_port_arg, construct_server_url, get_path_from_matches_or_current_path};

#[derive(Clone)]
pub struct TraceSubCommand {}

impl SubCommandUtil for TraceSubCommand {
    fn name(&self) -> &str { "trace" }

    fn alias(&self) -> &str { "" }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Trace directory handler ouput")
            .arg(construct_port_arg())
            .args(construct_directory_or_all_args().as_slice())
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches) {
        if let Some(server_url) = construct_server_url(sub_matches) {
            match connect_client(server_url) {
                Ok(client) => execute_trace(sub_matches, client),
                Err(e) => println!("{}", e)
            }
        }
        else {

        }
    }
}

fn execute_trace(sub_matches: &ArgMatches, mut client: HandlerServiceClient<tonic::transport::Channel>) {
    let mut directory_path = String::new();
    if !sub_matches.is_present("all") {
        let path = get_path_from_matches_or_current_path(sub_matches, "directory").unwrap();
        directory_path = path.into_os_string().into_string().unwrap();
    }
    let response = client.trace_handler(TraceHandlerRequest {
        directory_path,
    });
    let response = block_on(response);
    match response {
        Ok(response) => {
            let mut stream = response.into_inner();
            while let Some(response) = block_on(stream.message()).unwrap() {
                println!("{:?}", response);
            }
        }
        Err(e) => println!("{}", e)
    }
}