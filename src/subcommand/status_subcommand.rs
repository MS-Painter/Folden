use futures::executor::block_on;
use clap::{App, Arg, ArgMatches};
use tonic::transport::{Channel, Error as TransportError};

use folder_handler::handlers_json::HandlersJson;
use crate::subcommand::subcommand::SubCommandUtil;
use generated_types::inter_process_client::InterProcessClient;
use generated_types::GetDirectoryStatusRequest;

pub struct StatusSubCommand {
    handlers_json: HandlersJson
}

impl SubCommandUtil for StatusSubCommand {
    fn new(handlers_json: HandlersJson) -> Self {
        Self { handlers_json }
    }

    fn name(&self) -> &str {
        "status"
    }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Fun folder usage in current working directory")
            .arg(Arg::with_name("debug").short("d")
                .help("print debug information verbosely"))
            .arg(Arg::with_name("directory")
                .required(false)
                .empty_values(false)
                .takes_value(true)
                .validator_os(StatusSubCommand::is_existing_directory_validator))
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches, client_connect_future: impl futures::Future<Output = Result<InterProcessClient<Channel>, TransportError>>) {        
        let path = StatusSubCommand::get_path_from_matches_or_current_path(sub_matches, "directory").unwrap();
        let mut client = block_on(client_connect_future).unwrap();
        let response = client.get_directory_status(GetDirectoryStatusRequest {
            directory_path: String::from(path.as_os_str().to_str().unwrap())
        });
        let response = block_on(response).unwrap().into_inner();
        println!("{:?}", response.message);
    }
}