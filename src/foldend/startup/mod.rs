#[cfg(windows)]
pub mod windows;

use std::{fs, path::PathBuf};
use std::collections::HashMap;
use std::{convert::TryFrom, sync::Arc};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

use tonic::Request;
use tokio::sync::RwLock;
use tonic::transport::Server as TonicServer;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand, crate_version};
use tracing_subscriber::fmt::format;

use crate::config::Config;
use crate::server::Server;
use crate::mapping::Mapping;
use generated_types::{DEFAULT_PORT_STR, StartHandlerRequest, handler_service_server::{HandlerService, HandlerServiceServer}};


fn construct_app<'a, 'b>() -> App<'a, 'b> {
    App::new("Folden Service")
        .version(crate_version!())
        .about("Folden background manager service")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("run").about("Startup Folden server")
            .arg(Arg::with_name("config").short("c").long("config")
                .help("Startup config file")
                .required(true).
                empty_values(false)
                .takes_value(true))
            .arg(Arg::with_name("mapping").short("m").long("mapping")
                .required(false).
                empty_values(false)
                .takes_value(true)
                .help("Startup mapping file"))
            .arg(Arg::with_name("port").short("p").long("port")
                .default_value(DEFAULT_PORT_STR)
                .empty_values(false)
                .takes_value(true))
            .arg(Arg::with_name("log").short("l").long("log")
                .default_value(DEFAULT_PORT_STR)
                .empty_values(false)
                .takes_value(true)))
        .subcommand(SubCommand::with_name("logs").about("Interact with local server side logs")
            .arg(Arg::with_name("clear").long("clear")
                .required(true)
                .takes_value(false)
                .conflicts_with("view")
                .help("Wipe saved logs"))
            .arg(Arg::with_name("view").long("view")
                .required(true)
                .takes_value(false)
                .conflicts_with("clear")
                .help("View saved logs")))
}

async fn startup_handlers(server: &Server) -> () {
    let mapping = server.mapping.read().await;
    let handler_requests: Vec<StartHandlerRequest> = mapping.directory_mapping.iter()
    .filter_map(|(directory_path, handler_mapping)| {
        if handler_mapping.is_auto_startup {
            Some(StartHandlerRequest {
                directory_path: directory_path.to_string(),
            })
        }
        else {
            None
        }
    }).collect();
    drop(mapping);
    for request in handler_requests {
        match server.start_handler(Request::new(request.clone())).await {
            Ok(response) => {
                println!("{:?}", response.into_inner().states_map);
            }
            Err(err) => {
                println!("Handler [DOWN] - {:?}\n Error - {:?}", request.directory_path, err);
            }
        }
    }
}

async fn startup_server(config: Config, mapping: Mapping) -> Result<(), Box<dyn std::error::Error>> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), config.port);
    let server = Server {
        config: Arc::new(config),
        mapping: Arc::new(RwLock::new(mapping)),
    };

    startup_handlers(&server).await; // Handlers are raised before being able to accept client calls.

    println!("Server up on port {}", socket.port());
    TonicServer::builder()
        .add_service(HandlerServiceServer::new(server))
        .serve(socket)
        .await?;
    Ok(())
}

fn get_mapping(config: &Config) -> Mapping {
    let mapping = Mapping {
        directory_mapping: HashMap::new()
    };
    let mapping_file_path = &config.mapping_state_path;
    match fs::read(mapping_file_path) {
        Ok(mapping_file_data) => {
            match Mapping::try_from(mapping_file_data) {
                Ok(read_mapping) => read_mapping,
                Err(_) => {
                    println!("Mapping file invalid / empty");
                    mapping
                }
            }
        }
        Err(err) => {
            println!("Mapping file not found. Creating file - {:?}", mapping_file_path);
            match err.kind() {
                std::io::ErrorKind::NotFound => {
                    let mapping_file_parent_path = mapping_file_path.parent().unwrap();
                    if !mapping_file_parent_path.exists() {
                        fs::create_dir_all(mapping_file_parent_path).unwrap();
                    }
                    match fs::write(mapping_file_path,  b"") {
                        Ok(_) => mapping,
                        Err(err) => panic!("{}", err)
                    }
                }
                err => panic!("{:?}", err)
            }
        }
    }
}

fn get_config(file_path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    match fs::read(&file_path) {
        Ok(data) => Ok(Config::try_from(data)?),
        Err(e) => Err(e)?
    }
}

fn modify_config(config: &mut Config, sub_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(mapping_file_path) = sub_matches.value_of("mapping") {
        config.mapping_state_path.clone_from(&PathBuf::from(mapping_file_path));
    }
    if let Some(port) = sub_matches.value_of("port") {
        config.port = port.parse()?;
    }
    Ok(())
}

pub async fn main_service_runtime() -> Result<(), Box<dyn std::error::Error>> {
    let app = construct_app();
    let matches = app.get_matches();
    if let Some(sub_matches) = matches.subcommand_matches("run") {
        match sub_matches.value_of("config") {
            Some(path) => {
                let config_file_path = PathBuf::from(path);
                match get_config(&config_file_path) {
                    Ok(mut config) => {
                        modify_config(&mut config, sub_matches)?;
                        config.save(&config_file_path).unwrap();
                        let mapping = get_mapping(&config);
                        startup_server(config, mapping).await?;
                    }
                    Err(err) => {
                        println!("Invalid config file:{path:?}\nError:{error}\nCreating default config", path=&config_file_path, error=err);
                        let mut config = Config::default();
                        modify_config(&mut config, sub_matches)?;
                        config.save(&config_file_path).unwrap();
                        let mapping = get_mapping(&config);
                        startup_server(config, mapping).await?;
                    }
                }
            }
            None => Err("Config path not provided")?
        }
    }
    else if let Some(sub_matches) = matches.subcommand_matches("logs") {
        if sub_matches.value_of("view").is_some() {

        }
        else if sub_matches.value_of("clear").is_some() {

        }
    }
    Ok(())
}