use folder_handler::handlers_json::HandlersJson;
use clap::{App, Arg, ArgMatches};
use crate::subcommand::subcommand::SubCommandUtil;
use std::env;
use std::path::PathBuf;

pub struct GenerateSubCommand {
    handlers_json: HandlersJson
}

impl GenerateSubCommand {
    fn generate_config_path(handler_match: &str, path: Option<&str>) -> PathBuf {
        match path {
            None => {
                let mut path_buf = env::current_dir().unwrap();
                path_buf.push(handler_match);
                path_buf.set_extension("toml");
                path_buf
            }
            Some(path) => {
                let mut path_buf = PathBuf::from(path);
                if path_buf.is_dir() {
                    path_buf.push(handler_match);
                    path_buf.set_extension("toml");
                }
                path_buf
            }
        }
    }
}

impl SubCommandUtil for GenerateSubCommand {
    fn new(handlers_json: HandlersJson) -> Self {
        Self { handlers_json }
    }

    fn name(&self) -> &str { "generate" }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Generate default handler config for input registered handler")
            .arg(Arg::with_name("debug")
                .short("d")
                .help("print debug information verbosely"))
            .arg(Arg::with_name("handler")
                .required(true)
                .empty_values(false)
                .case_insensitive(true)
                .possible_values(self.handlers_json.get_handler_types().as_slice()))
            .arg(Arg::with_name("path")
                .required(false))
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches, client_connect_future: impl futures::Future<Output = Result<generated_types::inter_process_client::InterProcessClient<tonic::transport::Channel>, tonic::transport::Error>>) {
        let handler_match = sub_matches.value_of("handler").unwrap();
        let path_match = match sub_matches.value_of("path") {
            None => GenerateSubCommand::generate_config_path(handler_match, None),
            Some(path) => GenerateSubCommand::generate_config_path(handler_match, Some(path))
        };
        match self.handlers_json.get_handler_by_name(&handler_match) {
            Ok(handler) => handler.generate_config(path_match.as_ref()).unwrap(),
            Err(e) => panic!(e)
        }
    }
}
