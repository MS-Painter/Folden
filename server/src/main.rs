use std::thread;
use std::sync::mpsc;

use clap::{App, AppSettings, Arg, SubCommand};

const DEFAULT_CONFIG_PATH: &str = "default.conf";


fn startup_server() {
    //let (tx, rx) = mpsc::channel();
    unimplemented!();
}

fn main() {
    let app = App::new("Folden Server")
        .version("0.1")
        .about("Folden background manager")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true))
        .subcommand(SubCommand::with_name("run")
            .help("Startup Folden server"));
    let matches = app.get_matches();
    let config = matches.value_of("config").unwrap_or(DEFAULT_CONFIG_PATH);
    println!("Value for config: {}", config);
    if let Some(_) = matches.subcommand_matches("run") {
        startup_server();
    }
}