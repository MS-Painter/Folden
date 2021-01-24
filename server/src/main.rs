use std::{fs, sync::Arc, thread};
use std::sync::mpsc;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

use clap::{App, AppSettings, Arg, SubCommand};
use tonic::{Request, Response};
use tonic::transport::Server as TonicServer;

mod config;
use config::Config;
mod database_access;
use database_access::database_access::establish_connection;
use generated_types::inter_process_server::{InterProcess, InterProcessServer};
use generated_types::{
    RegisterToDirectoryRequest, RegisterToDirectoryResponse,
    GetDirectoryStatusRequest, GetDirectoryStatusResponse
};

const DEFAULT_CONFIG_PATH: &str = "default_config.toml";

struct Server {
    config: Arc<Config>,
    db: Arc<rocksdb::DB>
}

#[tonic::async_trait]
impl InterProcess for Server {
    async fn register_to_directory(&self, request:Request<RegisterToDirectoryRequest>) ->
    Result<Response<RegisterToDirectoryResponse>,tonic::Status> {
        let request = request.into_inner();
        println!("{}", request.full_directory_path);
        println!("{}", request.handler_type_name);

        Ok(Response::new(RegisterToDirectoryResponse {
            message: "".to_string(),
        }))
    }

    async fn get_directory_status(&self, request:Request<GetDirectoryStatusRequest>) ->
    Result<Response<GetDirectoryStatusResponse>,tonic::Status> {
        let request = request.into_inner();
        println!("{}", request.full_directory_path);

        Ok(Response::new(GetDirectoryStatusResponse {
            message: "".to_string(),
        }))
    }
}

async fn startup_server(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    //let (tx, rx) = mpsc::channel();
    println!("{:?}", config);
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

    let server = Server{
        db: Arc::new(establish_connection(&config.db_path).unwrap()),
        config: Arc::new(config)
    };

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
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true))
        .subcommand(SubCommand::with_name("run")
            .help("Startup Folden server"));
    let matches = app.get_matches();
    let config_file_path = matches.value_of("config").unwrap_or(DEFAULT_CONFIG_PATH);
    let config_file_data = fs::read(config_file_path).unwrap();
    let config = Config::from(config_file_data);
    if let Some(_) = matches.subcommand_matches("run") {
        startup_server(config).await?;
    }
    Ok(())
}