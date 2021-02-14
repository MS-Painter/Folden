use std::error::Error;

extern crate clap;
use clap::{App, AppSettings};

mod subcommand;
use subcommand::cli::Cli;

const GRPC_URL_BASE: &str = "http://localhost:8080/";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut cli = Cli::new( 
        App::new("Folden")
            .version("0.1")
            .about("System-wide folder event handling")
            .setting(AppSettings::SubcommandRequiredElseHelp)
        );
    cli.execute(GRPC_URL_BASE);
    Ok(())
}
