use std::{env, ops::Deref, path::PathBuf};

use clap::{App, Arg, ArgMatches};
use strum::VariantNames;

use super::subcommand_utils::SubCommandUtil;
use pipelines::{actions::PipelineActions, event::EVENT_TYPES, pipeline_config::PipelineConfig};

#[derive(Clone)]
pub struct GenerateSubCommand;

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
}

impl SubCommandUtil for GenerateSubCommand {
    fn name(&self) -> &str {
        "generate"
    }

    fn alias(&self) -> &str {
        "gen"
    }

    fn requires_connection(&self) -> bool {
        false
    }

    fn construct_subcommand(&self) -> App {
        self.create_instance()
            .about("Generate default handler pipeline config")
            .arg(
                Arg::with_name("events")
                    .long("events")
                    .required(false)
                    .multiple(true)
                    .empty_values(false)
                    .case_insensitive(true)
                    .possible_values(&EVENT_TYPES),
            )
            .arg(
                Arg::with_name("actions")
                    .long("actions")
                    .required(false)
                    .multiple(true)
                    .empty_values(false)
                    .case_insensitive(true)
                    .possible_values(&PipelineActions::VARIANTS),
            )
            .arg(
                Arg::with_name("path")
                    .required(false)
                    .help("File path. Leave empty to generate with default name."),
            )
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches) {
        let events = sub_matches.values_of("events");
        let actions = sub_matches.values_of("actions");
        let path = GenerateSubCommand::construct_config_path(
            "folden_pipeline",
            sub_matches.value_of("path"),
        );
        let config = PipelineConfig::default_new(events, actions);
        config.generate_config(path.deref()).unwrap();
    }

    fn subcommand_connection_runtime(
        &self,
        sub_matches: &ArgMatches,
        _client: generated_types::handler_service_client::HandlerServiceClient<
            tonic::transport::Channel,
        >,
    ) {
        self.subcommand_runtime(sub_matches);
    }
}
