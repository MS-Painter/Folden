use std::{collections::HashMap, convert::TryFrom, fs, sync::Arc};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

use tokio::sync::RwLock;
use tonic::{Request, transport::Server as TonicServer};
use clap::{App, AppSettings, Arg, SubCommand};

mod server;
use server::Server;
mod config;
use config::{Config, MappingStatusStrategy};
mod mapping;
use mapping::Mapping;
use folder_handler::handlers_json::HandlersJson;
use generated_types::{StartHandlerRequest, inter_process_server::{InterProcess, InterProcessServer}};

const DEFAULT_CONFIG_PATH: &str = "default_config.toml";
const DEFAULT_MAPPING_STATE_PATH: &str = "default_mapping.toml";

async fn handle_mapping_strategy(server: &Server) -> () {
    match server.config.mapping_status_strategy {
        MappingStatusStrategy::None => {}
        MappingStatusStrategy::Save => {}
        MappingStatusStrategy::Continue => {
            let mapping = server.mapping.read().await;
            let mut handler_requests: Vec<StartHandlerRequest> = Vec::new();
            for directory_path in mapping.directory_mapping.keys() {
                let request = StartHandlerRequest {
                    directory_path: directory_path.clone(),
                };
                handler_requests.push(request);
            }
            drop(mapping); // Free lock to complete server requests.
            for request in handler_requests {
                let response = server.start_handler(Request::new(request.clone())).await;
                match response {
                    Ok(_) => {
                        println!("Handler {:?} - up", request.directory_path);
                    }
                    Err(err) => {
                        println!("Handler {:?} - down\n Error: {:?}", request.directory_path, err);
                    }
                }
            }
        }
    }
}

async fn startup_server(config: Config, mapping: Mapping, handlers_json: HandlersJson) -> Result<(), Box<dyn std::error::Error>> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    
    let server = Server {
        config: Arc::new(config),
        mapping: Arc::new(RwLock::new(mapping)),
        handlers_json: Arc::new(handlers_json), 
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
    let app = App::new("Folden Server")
        .version("0.1")
        .about("Folden background manager")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(Arg::with_name("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true))
        .arg(Arg::with_name("mapping")
            .value_name("FILE")
            .help("Provide custom path for existing saved mapping")
            .required(false)
            .takes_value(true))
        .subcommand(SubCommand::with_name("run")
            .help("Startup Folden server"));
    let matches = app.get_matches();
    let config_file_path = matches.value_of("config").unwrap_or(DEFAULT_CONFIG_PATH);
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
                    let mapping_file_path = matches.value_of("mapping").unwrap_or(DEFAULT_MAPPING_STATE_PATH);
                    match fs::read(mapping_file_path) {
                        Ok(mapping_file_data) => {
                            match Mapping::try_from(mapping_file_data) {
                                Ok(read_mapping) => {
                                    mapping = read_mapping;
                                }
                                Err(_) => {
                                    println!("Mapping file invalid");
                                }
                            }
                        }
                        Err(err) => {
                            println!("Mapping file not found. Creating file - {:?}", mapping_file_path);
                            match err.kind() {
                                std::io::ErrorKind::NotFound => {
                                    match fs::write(mapping_file_path,  b"") {
                                        Ok(_) => {}
                                        Err(err) => panic!(err)
                                    }
                                }
                                err => panic!(err)
                            }
                        }
                    }    
                }
            }
            let handlers_json = HandlersJson::new();
            if let Some(_) = matches.subcommand_matches("run") {
                startup_server(config, mapping, handlers_json).await?;
            }
            Ok(())
        }
        Err(err) => {
            panic!("Invalid config file: {path}\nError:{error}", path=config_file_path, error=err)
        }
    }
}