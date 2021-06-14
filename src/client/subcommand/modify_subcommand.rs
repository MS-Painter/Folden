use clap::{App, Arg, ArgMatches};
use futures::executor::block_on;

use super::subcommand_utils::{
    construct_directory_or_all_args, construct_startup_type_arg,
    get_path_from_matches_or_current_path, SubCommandUtil,
};
use folden::shared_utils::construct_port_arg;
use generated_types::{handler_service_client::HandlerServiceClient, ModifyHandlerRequest};

#[derive(Clone)]
pub struct ModifySubCommand;

impl SubCommandUtil for ModifySubCommand {
    fn name(&self) -> &str {
        "modify"
    }

    fn alias(&self) -> &str {
        "mod"
    }

    fn requires_connection(&self) -> bool {
        true
    }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Modify directory handler")
            .arg(
                Arg::with_name("description")
                    .long("description")
                    .visible_alias("desc")
                    .required(false)
                    .takes_value(true),
            )
            .arg(construct_port_arg())
            .arg(construct_startup_type_arg())
            .args(construct_directory_or_all_args().as_slice())
    }

    fn subcommand_connection_runtime(
        &self,
        sub_matches: &ArgMatches,
        mut client: HandlerServiceClient<tonic::transport::Channel>,
    ) {
        let is_auto_startup = sub_matches
            .value_of("startup")
            .map(|value| value.to_lowercase() == "auto");
        let modify_description = sub_matches
            .value_of("description")
            .map(|description| description.to_string());
        let mut directory_path = String::new();
        if !sub_matches.is_present("all") {
            let path = get_path_from_matches_or_current_path(sub_matches, "directory").unwrap();
            directory_path = path.into_os_string().into_string().unwrap();
        } else if let Some(path) = sub_matches.value_of_os("directory") {
            directory_path = path.to_os_string().into_string().unwrap();
        }
        let response = client.modify_handler(ModifyHandlerRequest {
            directory_path,
            is_auto_startup,
            modify_description,
        });
        let response = block_on(response);
        match response {
            Ok(_) => {}
            Err(e) => println!("{}", e),
        }
    }
}
