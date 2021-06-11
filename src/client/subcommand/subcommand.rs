use std::{env, ffi::{OsStr, OsString}, option::Option, path::Path};

use dyn_clone::DynClone;
use clap::{App, Arg, ArgMatches, SubCommand};
use cli_table::{self, Cell, CellStruct, Table, TableStruct, print_stdout};

use generated_types::{DEFAULT_PORT_STR, HandlerStatesMapResponse, HandlerSummaryMapResponse, handler_service_client::HandlerServiceClient};

const STARTUP_TYPES: [&str; 2] = ["auto", "manual"];

pub trait SubCommandUtil: DynClone {
    fn name(&self) -> &str;

    fn alias(&self) -> &str;

    fn requires_connection(&self) -> bool;
    
    fn construct_subcommand(&self) -> App;
    
    fn subcommand_runtime(&self, sub_matches: &ArgMatches, server_url: Option<String>);
    
    fn create_instance(&self) -> App {
        if self.alias().is_empty() {
            SubCommand::with_name(self.name())
        }
        else {
            SubCommand::with_name(self.name()).visible_alias(self.alias())
        }
    }

    fn subcommand_matches<'a>(&self, matches: &'a ArgMatches) -> Option<&clap::ArgMatches<'a>> {
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
        self.0.as_slice().into_iter()
        .map(|item| item.construct_subcommand())
        .collect()
    }
}

impl IntoIterator for SubCommandCollection {
    type Item = Box<dyn SubCommandUtil>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub fn connect_client<D>(dst: D)-> Result<HandlerServiceClient<tonic::transport::Channel>, tonic::transport::Error> 
where
    D: std::convert::TryInto<tonic::transport::Endpoint>,
    D::Error: Into<tonic::codegen::StdError> {
    let client_connect_future = HandlerServiceClient::connect(dst);
    futures::executor::block_on(client_connect_future)
}

pub fn construct_port_arg<'a, 'b>() -> Arg<'a, 'b>{
    Arg::with_name("port").short("p").long("port")
        .default_value(DEFAULT_PORT_STR)
        .empty_values(false)
        .takes_value(true)
}

pub fn construct_directory_or_all_args<'a, 'b>() -> Vec<Arg<'a, 'b>>{
    vec!(Arg::with_name("directory").long("directory").visible_alias("dir")
            .required(false)
            .empty_values(false)
            .takes_value(true)
            .validator_os(is_existing_directory_validator),
        Arg::with_name("all").long("all")
            .help("Apply on all registered directory handlers")
            .required(false)
            .takes_value(false)
            .conflicts_with("directory"))
}

pub fn construct_startup_type_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("startup").long("startup").visible_alias("up")
        .help("Set if handler starts on service startup")
        .required(false)
        .takes_value(true)
        .case_insensitive(true)
        .possible_values(&STARTUP_TYPES)
}

pub fn construct_simple_output_arg<'a, 'b>() -> Arg<'a, 'b>{
    Arg::with_name("simple").long("simple").visible_alias("smpl")
        .help("Output in simplified format")
        .takes_value(false)
}

pub fn get_path_from_matches_or_current_path(sub_matches: &ArgMatches, match_name: &str) -> Result<std::path::PathBuf, std::io::Error> {
    match sub_matches.value_of(match_name) {
        Some(directory_match) => {
            Path::new(directory_match).canonicalize()
        }
        None => {
            env::current_dir().unwrap().canonicalize()
        }
    }
}

pub fn construct_server_url(sub_matches: &ArgMatches) -> Option<String> {
    if let Some(value) = sub_matches.value_of("port") {
        return Some(format!("http://localhost:{}/", value))
    }
    None
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

fn get_handler_states_table(states_map_response: HandlerStatesMapResponse) -> TableStruct {
    states_map_response.states_map.into_iter()
        .map(|(dir, state)| 
        vec![dir.cell(), state.is_alive.cell(), state.message.cell()])
        .collect::<Vec<Vec<CellStruct>>>()
        .table()
        .title(vec![
            "Path".cell(),
            "Alive".cell(),
            "Message".cell(),
        ])
}

fn get_handler_summaries_table(summary_map_response: HandlerSummaryMapResponse) -> TableStruct {
    summary_map_response.summary_map.into_iter()
        .map(|(dir, summary)| 
        vec![dir.cell(), summary.description.cell(), summary.is_alive.cell(), (if summary.is_auto_startup {"auto"} else {"manual"}).cell()])
        .collect::<Vec<Vec<CellStruct>>>()
        .table()
        .title(vec![
            "Path".cell(),
            "Description".cell(),
            "Alive".cell(),
            "Startup".cell(),
        ])
}

pub fn print_handler_states(states_map_response: HandlerStatesMapResponse, sub_matches: &ArgMatches) {
    if sub_matches.is_present("simple") {
        for (dir, state) in states_map_response.states_map {
            println!("
            {}
            Alive: {}
            Message: {}", 
            dir, state.is_alive, state.message);
        }
    }
    else {
        let table = get_handler_states_table(states_map_response);
        print_stdout(table).unwrap();
    }
}

pub fn print_handler_summaries(summary_map_response: HandlerSummaryMapResponse, sub_matches: &ArgMatches) {
    if sub_matches.is_present("simple") {
        for (dir, summary) in summary_map_response.summary_map {
            println!("
            {}
            Description: {}
            Alive: {}
            Startup: {}", 
            dir, summary.description, summary.is_alive, if summary.is_auto_startup {"auto"} else {"manual"});
        }
    }
    else {
        let table = get_handler_summaries_table(summary_map_response);
        print_stdout(table).unwrap();
    }
}
