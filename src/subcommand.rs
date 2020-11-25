use std::{env, option::Option};

use clap::{App, SubCommand, Arg, ArgMatches};

extern crate folder_handler;
use folder_handler::handlers_json::HandlersJson;

pub trait SubCommandUtil {
    fn new(handlers_json: HandlersJson) -> Self;
    fn name(&self) -> &str;
    fn construct_subcommand(&self) -> App;
    fn subcommand_runtime(&self, sub_matches: &ArgMatches);
    fn create_instance(&self) -> App {
        SubCommand::with_name(self.name())
    }
    fn subcommand_matches<'a>(&self, matches: &'a ArgMatches) -> Option<&clap::ArgMatches<'a>> {
        matches.subcommand_matches(self.name())
    }
}

pub struct ThisSubCommand {
    handlers_json: HandlersJson
}

impl SubCommandUtil for ThisSubCommand {
    fn new(handlers_json: HandlersJson) -> Self {
        Self{ handlers_json }
    }

    fn name(&self) -> &str {
        "this"
    }
    
    fn construct_subcommand(&self) -> App{
        self.create_instance()
        .about("Fun folder usage in current working directory")
        .arg(Arg::with_name("debug").short("d")
        .help("print debug information verbosely"))
        .arg(Arg::with_name("register").short("r").long("register").value_name("FILE").takes_value(true)
        .help("Register new handler type with provided override rust module"))
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches){
        let path = env::current_dir().unwrap();
        println!("The current directory is {}", path.display());
        println!("{:?}", sub_matches);
        if sub_matches.args.contains_key("register") {

        }
    }
}

pub struct ListSubCommand {
    handlers_json: HandlersJson
}

impl SubCommandUtil for ListSubCommand {
    fn new(handlers_json: HandlersJson) -> Self {
        Self{ handlers_json }
    }

    fn name(&self) -> &str {
        "list"
    }
    
    fn construct_subcommand(&self) -> App{
        self.create_instance()
        .about("Details on usage of fun folder across entire filesystem")
        .arg(Arg::with_name("debug").short("d")
        .help("print debug information verbosely"))
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches){
        println!("{:?}", &sub_matches.args.len())
    }
}

pub struct GenerateSubCommand{
    handlers_json: HandlersJson
}

impl SubCommandUtil for GenerateSubCommand {
    fn new(handlers_json: HandlersJson) -> Self {
        Self{ handlers_json }
    }

    fn name(&self) -> &str {"generate"}

    fn construct_subcommand(&self) -> App{
        self.create_instance()
        .about("Generate default handler config for input registered handler")
        .arg(Arg::with_name("debug")
            .short("d")
            .help("print debug information verbosely"))
        .arg(Arg::with_name("handler")
            .takes_value(true)
            .possible_values(self.handlers_json.get_handler_types().as_slice()))
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches) {
        println!("{:?}", &sub_matches.args)
    }
}
