use std::error::Error;

extern crate clap;
use clap::{App, AppSettings};

mod subcommand;
use subcommand::cli::{Cli, excute};

const GRPC_URL_BASE: &str = "http://localhost:8080/";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::new( 
        App::new("Folden")
            .version("0.1")
            .about("System-wide folder event handling")
            .setting(AppSettings::SubcommandRequiredElseHelp)
        );
    excute(cli, GRPC_URL_BASE);
    Ok(())
}
