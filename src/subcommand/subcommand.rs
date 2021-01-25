use std::option::Option;

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
}
