use std::{
    env,
    option::Option,
    path::{Path, PathBuf},
};

use clap::{App, Arg, ArgMatches, SubCommand, builder::TypedValueParser};
use cli_table::{self, print_stdout, Cell, CellStruct, Table, TableStruct};
use dyn_clone::DynClone;

use generated_types::{
    handler_service_client::HandlerServiceClient, HandlerStatesMapResponse,
    HandlerSummaryMapResponse,
};

const STARTUP_TYPES: [&str; 2] = ["auto", "manual"];

pub trait SubCommandUtil: DynClone {
    fn name(&self) -> &str;

    fn alias(&self) -> &str;

    fn requires_connection(&self) -> bool;

    fn construct_subcommand(&self) -> App;

    fn subcommand_runtime(&self, _sub_matches: &ArgMatches) {
        panic!("Command execution without connection unsupported")
    }

    fn subcommand_connection_runtime(
        &self,
        sub_matches: &ArgMatches,
        client: HandlerServiceClient<tonic::transport::Channel>,
    );

    fn create_instance(&self) -> App {
        if self.alias().is_empty() {
            SubCommand::with_name(self.name())
        } else {
            SubCommand::with_name(self.name()).visible_alias(self.alias())
        }
    }

    fn subcommand_matches<'a>(&'a self, matches: &'a ArgMatches) -> Option<&clap::ArgMatches> {
        matches.subcommand_matches(self.name())
    }
}

dyn_clone::clone_trait_object!(SubCommandUtil);

#[derive(Clone)]
pub struct SubCommandCollection(Vec<Box<dyn SubCommandUtil>>);

impl SubCommandCollection {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, elem: Box<dyn SubCommandUtil>) {
        self.0.push(elem);
    }

    pub fn collect_as_apps(&self) -> Vec<App> {
        self.0
            .as_slice()
            .iter()
            .map(|item| item.construct_subcommand())
            .collect()
    }
}

impl Default for SubCommandCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for SubCommandCollection {
    type Item = Box<dyn SubCommandUtil>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub fn connect_client<D>(
    dst: D,
) -> Result<HandlerServiceClient<tonic::transport::Channel>, tonic::transport::Error>
where
    D: std::convert::TryInto<tonic::transport::Endpoint>,
    D::Error: Into<tonic::codegen::StdError>,
{
    let client_connect_future = HandlerServiceClient::connect(dst);
    futures::executor::block_on(client_connect_future)
}

pub fn construct_directory_or_all_args<'a>() -> Vec<Arg<'a>> {
    vec![
        Arg::with_name("directory")
            .long("directory")
            .visible_alias("dir")
            .required(false)
            .empty_values(false)
            .takes_value(true)
            .value_parser(clap::builder::ValueParser::new(ExistingDirectoryValueParser::new())),
        Arg::with_name("all")
            .long("all")
            .help("Apply on all registered directory handlers")
            .required(false)
            .takes_value(false)
            .conflicts_with("directory"),
    ]
}

pub fn construct_startup_type_arg<'a>() -> Arg<'a> {
    Arg::with_name("startup")
        .long("startup")
        .visible_alias("up")
        .help("Set if handler starts on service startup")
        .required(false)
        .takes_value(true)
        .case_insensitive(true)
        .possible_values(&STARTUP_TYPES)
}

pub fn construct_simple_output_arg<'a>() -> Arg<'a> {
    Arg::with_name("simple")
        .long("simple")
        .visible_alias("smpl")
        .help("Output in simplified format")
        .takes_value(false)
}

pub fn get_path_from_matches_or_current_path(
    sub_matches: &ArgMatches,
    match_name: &str,
) -> Result<PathBuf, std::io::Error> {
    match sub_matches.get_one::<PathBuf>(match_name) {
        Some(directory_match) => directory_match.as_path().canonicalize(),
        None => env::current_dir().unwrap().canonicalize(),
    }
}

pub fn construct_server_url(sub_matches: &ArgMatches) -> Option<String> {
    if let Some(value) = sub_matches.value_of("port") {
        return Some(format!("http://localhost:{}/", value));
    }
    None
}

#[derive(Copy, Clone, Debug)]
pub struct ExistingDirectoryValueParser {}

impl ExistingDirectoryValueParser {
    pub fn new() -> Self {
        Self {}
    }
}

impl TypedValueParser for ExistingDirectoryValueParser {
    type Value = PathBuf;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let path = Path::new(value).canonicalize()?;
        if path.is_dir() && path.exists() {
            Ok(path)
        } else {
            Err(clap::Error::raw(clap::ErrorKind::InvalidValue, "Input value isn't a directory"))
        }
    }
}

impl Default for ExistingDirectoryValueParser {
    fn default() -> Self {
        Self::new()
    }
}

fn get_handler_states_table(states_map_response: HandlerStatesMapResponse) -> TableStruct {
    states_map_response
        .states_map
        .into_iter()
        .map(|(dir, state)| vec![dir.cell(), state.is_alive.cell(), state.message.cell()])
        .collect::<Vec<Vec<CellStruct>>>()
        .table()
        .title(vec!["Path".cell(), "Alive".cell(), "Message".cell()])
}

fn get_handler_summaries_table(summary_map_response: HandlerSummaryMapResponse) -> TableStruct {
    summary_map_response
        .summary_map
        .into_iter()
        .map(|(dir, summary)| {
            vec![
                dir.cell(),
                summary.description.cell(),
                summary.is_alive.cell(),
                (if summary.is_auto_startup {
                    "auto"
                } else {
                    "manual"
                })
                .cell(),
            ]
        })
        .collect::<Vec<Vec<CellStruct>>>()
        .table()
        .title(vec![
            "Path".cell(),
            "Description".cell(),
            "Alive".cell(),
            "Startup".cell(),
        ])
}

pub fn print_handler_states(
    states_map_response: HandlerStatesMapResponse,
    sub_matches: &ArgMatches,
) {
    if sub_matches.is_present("simple") {
        for (dir, state) in states_map_response.states_map {
            println!(
                "
            {}
            Alive: {}
            Message: {}",
                dir, state.is_alive, state.message
            );
        }
    } else {
        let table = get_handler_states_table(states_map_response);
        print_stdout(table).unwrap();
    }
}

pub fn print_handler_summaries(
    summary_map_response: HandlerSummaryMapResponse,
    sub_matches: &ArgMatches,
) {
    if sub_matches.is_present("simple") {
        for (dir, summary) in summary_map_response.summary_map {
            println!(
                "
            {}
            Description: {}
            Alive: {}
            Startup: {}",
                dir,
                summary.description,
                summary.is_alive,
                if summary.is_auto_startup {
                    "auto"
                } else {
                    "manual"
                }
            );
        }
    } else {
        let table = get_handler_summaries_table(summary_map_response);
        print_stdout(table).unwrap();
    }
}
