use futures::executor::block_on;
use clap::{App, Arg, ArgMatches};
use tonic::transport::{Channel, Error as TransportError};

use folder_handler::handlers_json::HandlersJson;
use crate::subcommand::subcommand::SubCommandUtil;
use generated_types::inter_process_client::InterProcessClient;
use generated_types::StopHandlerRequest;

pub struct StopSubCommand  {
    handlers_json: HandlersJson
}

impl SubCommandUtil for StopSubCommand {
    fn new(handlers_json: HandlersJson) -> Self {
        Self { handlers_json }
    }

    fn name(&self) -> &str {
        "stop"
    }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Stop handler on directory")
            .arg(Arg::with_name("remove").long("remove")
            .required(false)
            .takes_value(false))
            .arg(Arg::with_name("debug").short("d")
                .help("print debug information verbosely"))
            .arg(Arg::with_name("directory")
                .required(false)
                .empty_values(false)
                .takes_value(true)
                .validator_os(StopSubCommand::is_existing_directory_validator))
        }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches, client_connect_future: impl futures::Future<Output = Result<InterProcessClient<Channel>, TransportError>>) {
        let is_handler_to_be_removed = sub_matches.is_present("remove");
        
        let path = StopSubCommand::get_path_from_matches_or_current_path(sub_matches, "directory").unwrap();
        
        let mut client = block_on(client_connect_future).unwrap();
        let response = client.stop_handler(StopHandlerRequest {
            directory_path: String::from(path.as_os_str().to_str().unwrap()),
            is_handler_to_be_removed,
        });
        let response = block_on(response).unwrap().into_inner();
        println!("{:?}", response.message);
    }
}