use std::fs;
use std::sync::Arc;
use std::convert::TryFrom;
use std::collections::HashMap;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

use tokio::sync::RwLock;
use clap::{App, AppSettings, Arg, SubCommand};
use tonic::{Request, transport::Server as TonicServer};

mod server;
use server::Server;
mod config;
use config::{Config, MappingStatusStrategy};
mod mapping;
use mapping::Mapping;
use generated_types::{StartHandlerRequest, inter_process_server::{InterProcess, InterProcessServer}};

fn construct_app<'a, 'b>() -> App<'a, 'b> {
    App::new("Folden Service")
        .version("0.1")
        .about("Folden background manager service")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("run")
            .help("Startup Folden server")
            .arg(Arg::with_name("config").short("c").long("config")
                .help("Startup config file")
                .required(true).
                empty_values(false)
                .takes_value(true)))
}

async fn handle_mapping_strategy(server: &Server) -> () {
    match server.config.mapping_status_strategy {
        MappingStatusStrategy::Continue => {
            let mapping = server.mapping.read().await;
            let mut handler_requests: Vec<StartHandlerRequest> = Vec::new();
            for directory_path in mapping.directory_mapping.keys() {
                handler_requests.push(StartHandlerRequest {
                    directory_path: directory_path.clone(),
                });
            }
            drop(mapping); // Free lock to complete server requests.
            for request in handler_requests {
                let response = server.start_handler(Request::new(request.clone())).await;
                match response {
                    Ok(_) => {
                        println!("Handler [RUNNING] - {:?}", request.directory_path);
                    }
                    Err(err) => {
                        println!("Handler [DOWN] - {:?}\n Error - {:?}", request.directory_path, err);
                    }
                }
            }
        }
        _ => {}
    }
}

async fn startup_server(config: Config, mapping: Mapping) -> Result<(), Box<dyn std::error::Error>> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    
    let server = Server {
        config: Arc::new(config),
        mapping: Arc::new(RwLock::new(mapping)),
    };

    handle_mapping_strategy(&server).await; // The handlers are raised before being able to accept client calls.

    TonicServer::builder()
        .add_service(InterProcessServer::new(server))
        .serve(socket)
        .await?;
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = construct_app();
    let matches = app.get_matches();
    if let Some(sub_matches) = matches.subcommand_matches("run") {
        let config_file_path = sub_matches.value_of("config").unwrap();
        match fs::read(config_file_path) {
            Ok(file_data) => {
                let config_file_data = file_data;
                let config = Config::from(config_file_data);
    
                let mut mapping = Mapping {
                    directory_mapping: HashMap::new()
                };
                match config.mapping_status_strategy {
                    MappingStatusStrategy::None => {}
                    MappingStatusStrategy::Save | MappingStatusStrategy::Continue => {
                        let mapping_file_path = &config.mapping_state_path;
                        match fs::read(mapping_file_path) {
                            Ok(mapping_file_data) => {
                                match Mapping::try_from(mapping_file_data) {
                                    Ok(read_mapping) => {
                                        mapping = read_mapping;
                                    }
                                    Err(_) => {
                                        println!("Mapping file invalid / empty");
                                    }
                                }
                            }
                            Err(err) => {
                                println!("Mapping file not found. Creating file - {:?}", mapping_file_path);
                                match err.kind() {
                                    std::io::ErrorKind::NotFound => {
                                        match fs::write(mapping_file_path,  b"") {
                                            Ok(_) => {}
                                            Err(err) => panic!("{}", err)
                                        }
                                    }
                                    err => panic!("{:?}", err)
                                }
                            }
                        }
                    }
                }
                startup_server(config, mapping).await?;
                Ok(())
            }
            Err(err) => {
                panic!("Invalid config file: {path}\nError:{error}", path=config_file_path, error=err)
            }
        }
    }
    else {
        Ok(())
    }
}