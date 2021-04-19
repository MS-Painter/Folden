use tonic::transport::Channel;
use futures::executor::block_on;
use clap::{App, Arg, ArgMatches};

use crate::subcommand::subcommand::SubCommandUtil;
use super::subcommand::{construct_directory_or_all_args, get_path_from_matches_or_current_path};
use generated_types::{ModifyHandlerRequest, HandlerStartupType};
use generated_types::inter_process_client::InterProcessClient;

#[derive(Clone)]
pub struct ModifySubCommand {}

impl SubCommandUtil for ModifySubCommand {
    fn name(&self) -> &str { "modify" }

    fn alias(&self) -> &str { "mod" }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Modify directory handler")
            .arg(Arg::with_name("debug").short("d")
                .help("print debug information verbosely"))
            .args(construct_directory_or_all_args().as_slice())
            .arg(Arg::with_name("manual_startup").long("manual_startup")
                .help("If present: Handler won't automatically start on service startup")
                .required(false)
                .takes_value(false))
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches, client: &mut InterProcessClient<Channel>) {
        let startup_type = if sub_matches.is_present("manual_startup") {HandlerStartupType::Off as i32} else {HandlerStartupType::On as i32};
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
            startup_type,
        });
        let response = block_on(response);
        println!("{:?}", response);
    }
}