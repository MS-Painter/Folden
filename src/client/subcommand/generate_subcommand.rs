use std::{env, ops::Deref, path::PathBuf};

use tonic::transport::Channel;
use clap::{App, Arg, ArgMatches, Values};
use workflows::{actions::WorkflowActions, event::WorkflowEvent, workflow_config::WorkflowConfig};

use crate::subcommand::subcommand::SubCommandUtil;
use generated_types::inter_process_client::InterProcessClient;

#[derive(Clone)]
pub struct GenerateSubCommand {}

impl GenerateSubCommand {
    fn construct_config_path(file_name: &str, path: Option<&str>) -> PathBuf {
        match path {
            None => {
                let mut path_buf = env::current_dir().unwrap();
                path_buf.push(file_name);
                path_buf.set_extension("toml");
                path_buf
            }
            Some(path) => {
                let mut path_buf = PathBuf::from(path);
                if path_buf.is_dir() {
                    path_buf.push(file_name);
                    path_buf.set_extension("toml");
                }
                path_buf
            }
        }
    }

    fn generate_config(path: PathBuf, events: Values, actions: Values) -> Result<(), std::io::Error> {
        let config = WorkflowConfig { 
            watch_recursive: false,
            apply_on_startup: false,
            panic_handler_on_error: false,
            event: WorkflowEvent::from(events),
            actions: WorkflowActions::defaults(actions),
        };
        config.generate_config(path.deref())
    }
}

impl SubCommandUtil for GenerateSubCommand {
    fn name(&self) -> &str { "generate" }

    fn alias(&self) -> &str { "gen" }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Generate default handler workflow config")
            .arg(Arg::with_name("debug")
                .short("d")
                .help("print debug information verbosely"))
            .arg(Arg::with_name("events").long("events")
                .required(true)
                .multiple(true)
                .empty_values(false)
                .case_insensitive(true)
                .possible_values(&["access", "create", "modify", "remove"]))
            .arg(Arg::with_name("actions").long("actions")
                .required(true)
                .multiple(true)
                .empty_values(false)
                .case_insensitive(true)
                .possible_values(&["movetodir", "runcmd"]))
            .arg(Arg::with_name("path")
                .required(false))
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches, _client: &mut InterProcessClient<Channel>) {
        let events = sub_matches.values_of("events").unwrap();
        let actions = sub_matches.values_of("actions").unwrap();
        let path = GenerateSubCommand::construct_config_path("folden_workflow",sub_matches.value_of("path"));
        GenerateSubCommand::generate_config(path, events, actions).unwrap();
    }
}