use futures::executor::block_on;
use clap::{App, Arg, ArgMatches};

use crate::subcommand::subcommand::SubCommandUtil;
use generated_types::{ModifyHandlerRequest, handler_service_client::HandlerServiceClient};
use super::subcommand::{construct_directory_or_all_args, construct_port_arg, construct_startup_type_arg, get_path_from_matches_or_current_path};

#[derive(Clone)]
pub struct ModifySubCommand {}

impl SubCommandUtil for ModifySubCommand {
    fn name(&self) -> &str { "modify" }

    fn alias(&self) -> &str { "mod" }

    fn requires_connection(&self) -> bool { true }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Modify directory handler")
            .arg(Arg::with_name("description").long("description").visible_alias("desc")
                .required(false)
                .takes_value(true))
            .arg(construct_port_arg())
            .arg(construct_startup_type_arg())
            .args(construct_directory_or_all_args().as_slice())
    }

    fn subcommand_connection_runtime(&self, sub_matches: &ArgMatches, mut client: HandlerServiceClient<tonic::transport::Channel>) {
        let is_auto_startup = match sub_matches.value_of("startup") {
            Some(value) => Some(if value.to_lowercase() == "auto" {true} else {false}),
            None => None
        };
        let modify_description = match sub_matches.value_of("description") {
            Some(description) => Some(description.to_string()),
            None => None
        };
        let mut directory_path = String::new();
        if !sub_matches.is_present("all") {
            let path = get_path_from_matches_or_current_path(sub_matches, "directory").unwrap();
            directory_path = path.into_os_string().into_string().unwrap();
        }
        else {
            match sub_matches.value_of_os("directory") {
                Some(path) => {
                    directory_path = path.to_os_string().into_string().unwrap();
                }
                None => {}
            }
        }
        let response = client.modify_handler(ModifyHandlerRequest {
            directory_path,
            is_auto_startup,
            modify_description,
        });
        let response = block_on(response);
        match response {
            Ok(_) => {}
            Err(e) => println!("{}", e)
        }
    }
}