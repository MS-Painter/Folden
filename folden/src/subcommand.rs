use std::{env, option::Option};

use clap::{App, SubCommand, Arg, ArgMatches};

pub trait SubCommandUtil {
    fn name(&self) -> &str;
    fn construct_subcommand(&self) -> App;
    fn subcommand_runtime(&self, sub_matches: &ArgMatches);
    fn subcommand_matches<'a>(&self, matches: &'a ArgMatches) -> Option<&clap::ArgMatches<'a>> {
        return matches.subcommand_matches(self.name());
    }
}

pub struct ThisSubCommand {
}

impl SubCommandUtil for ThisSubCommand {
    fn name(&self) -> &str {
        return "this";
    }
    
    fn construct_subcommand(&self) -> App{
        return SubCommand::with_name("this")
        .about("Fun folder usage in current working directory")
        .arg(Arg::with_name("debug").short("d")
        .help("print debug information verbosely"))
        .arg(Arg::with_name("register").short("r").long("register").value_name("FILE").takes_value(true)
        .help("Register new handler type with provided override rust module"));
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
}

impl SubCommandUtil for ListSubCommand {
    fn name(&self) -> &str {
        return "list";
    }
    
    fn construct_subcommand(&self) -> App{
        return SubCommand::with_name("list")
        .about("Details on usage of fun folder across entire filesystem")
        .arg(Arg::with_name("debug").short("d")
        .help("print debug information verbosely"));
    }

    fn subcommand_runtime(&self, sub_matches: &ArgMatches){
        println!("{:?}", &sub_matches.args.len());
    }
}