use std::env;
use std::path::Path;
use std::option::Option;
use std::ffi::{OsStr, OsString};

use futures::Future;
use tonic::transport::Channel;
use clap::{App, Arg, ArgMatches, SubCommand};

extern crate folder_handler;
use folder_handler::handlers_json::HandlersJson;
use generated_types::inter_process_client::InterProcessClient;

pub trait SubCommandUtil {
    fn new(handlers_json: HandlersJson) -> Self;
    
    fn name(&self) -> &str;
    
    fn construct_subcommand(&self) -> App;
    
    fn subcommand_runtime(&self, sub_matches: &ArgMatches, client_connect_future: impl Future<Output = Result<InterProcessClient<Channel>, tonic::transport::Error>>);
    
    fn create_instance(&self) -> App {
        SubCommand::with_name(self.name())
    }

    fn subcommand_matches<'a>(&self, matches: &'a ArgMatches) -> Option<&clap::ArgMatches<'a>> {
        matches.subcommand_matches(self.name())
    }

    fn construct_handler_arg<'a, 'b>(name: &'a str, handlers_json: &'b HandlersJson) -> Arg<'a, 'b> {
        Arg::with_name(name)
            .required(true)
            .empty_values(false)
            .case_insensitive(true)
            .possible_values(&handlers_json.get_handler_types().as_slice())
    }

    fn construct_directory_or_all_args<'a, 'b>() -> Vec<Arg<'a, 'b>>{
        vec!(Arg::with_name("directory").long("directory")
                .required(false)
                .empty_values(false)
                .takes_value(true)
                .validator_os(is_existing_directory_validator),
            Arg::with_name("all").long("all")
                .required(false)
                .takes_value(false)
                .conflicts_with("directory"))
    }

    fn get_path_from_matches_or_current_path(sub_matches: &ArgMatches, match_name: &str) -> Result<std::path::PathBuf, std::io::Error> {
        match sub_matches.value_of(match_name) {
            Some(directory_match) => {
                Path::new(directory_match).canonicalize()
            }
            None => {
                env::current_dir().unwrap().canonicalize()
            }
        }
    }

    
}

pub fn is_existing_directory_validator(val: &OsStr) -> Result<(), OsString> {
    let path = Path::new(val);
    if path.is_dir() && path.exists() {
        Ok(())
    }
    else {
        Err(OsString::from("Input value isn't a directory"))
    }
}
