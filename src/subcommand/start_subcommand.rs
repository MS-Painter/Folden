use std::env;

use futures::executor::block_on;
use clap::{App, Arg, ArgMatches};
use tonic::transport::{Channel, Error as TransportError};

use folder_handler::handlers_json::HandlersJson;
use crate::subcommand::subcommand::SubCommandUtil;
use generated_types::{StartHandlerRequest, inter_process_client::InterProcessClient};

pub struct StartSubCommand  {
    handlers_json: HandlersJson
}

impl SubCommandUtil for StartSubCommand {
    fn new(handlers_json: HandlersJson) -> Self {
        Self { handlers_json }
    }

    fn name(&self) -> &str {
        "start"
    }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Start handler on directory")
            .arg(Arg::with_name("debug").short("d")
                .help("print debug information verbosely"))
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches, client_connect_future: impl futures::Future<Output = Result<InterProcessClient<Channel>, TransportError>>) {        
        let path = env::current_dir().unwrap();
        
        let mut client = block_on(client_connect_future).unwrap();
        let response = client.start_handler(StartHandlerRequest {
            directory_path: String::from(path.as_os_str().to_str().unwrap()),
        });
        let response = block_on(response).unwrap().into_inner();
        println!("{:?}", response.message);
    }
}