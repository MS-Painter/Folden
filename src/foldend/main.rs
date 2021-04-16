use std::{fs, path::PathBuf};
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

#[cfg(windows)]
mod windows {
    use std::{ffi::OsString, time::Duration};
    
    use tokio::runtime::Runtime;
    pub use windows_service::Error;
    use windows_service::{
        define_windows_service, service_control_handler::{self, ServiceControlHandlerResult}, service_dispatcher,
        service::{ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType},
    };

    const SERVICE_NAME: &str = "Folden Service";

    define_windows_service!(ffi_service_main, service_main);
    
    fn service_main(arguments: Vec<OsString>) {
        if let Err(_e) = run_service(arguments) {
            // Handle errors in some way.
        }
    }

    pub fn run() -> Result<(), windows_service::Error> {
        // Register generated `ffi_service_main` with the system and start the service, blocking
        // this thread until the service is stopped.
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)
    }

    pub fn sync_main() -> Result<(), Box<dyn std::error::Error>> {
        let mut rt  = Runtime::new()?;
        // Spawn the root task
        rt.block_on(async {
            match  crate::main_service_runtime().await {
                Ok(res) => Ok(res),
                Err(e) => Err(e),
            }
        })
    }

    fn run_service(_arguments: Vec<OsString>) -> Result<(), windows_service::Error> {
        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Stop => {
                    // Handle stop event and return control back to the system.
                    ServiceControlHandlerResult::NoError
                }
                // All services must accept Interrogate even if it's a no-op.
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };
    
        // Register system service event handler
        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

        // Tell the system that the service is running now
        status_handle.set_service_status(ServiceStatus {
            // Should match the one from system service registry
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            // Accept stop events when running
            controls_accepted: ServiceControlAccept::STOP,
            // Used to report an error when starting or stopping only, otherwise must be zero
            exit_code: ServiceExitCode::Win32(0),
            // Only used for pending states, otherwise must be zero
            checkpoint: 0,
            // Only used for pending states, otherwise must be zero
            wait_hint: Duration::default(),
            process_id: None,
        })?;
    
        let exit_code = match sync_main() {
            Ok(_) => 0,
            Err(_) => 1
        };

        // Tell the system that service has stopped
        status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(exit_code),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        Ok(())
    }
}


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

fn get_mapping(config: &Config) -> Mapping {
    let mapping = Mapping {
        directory_mapping: HashMap::new()
    };
    match config.mapping_status_strategy {
        MappingStatusStrategy::None => mapping,
        MappingStatusStrategy::Save | MappingStatusStrategy::Continue => {
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
    }
}

async fn main_service_runtime() -> Result<(), Box<dyn std::error::Error>> {
    let app = construct_app();
    let matches = app.get_matches();
    if let Some(sub_matches) = matches.subcommand_matches("run") {
        let config_file_path = PathBuf::from(sub_matches.value_of("config").unwrap());
        match fs::read(&config_file_path) {
            Ok(file_data) => {
                let config_file_data = file_data;
                let config = match Config::try_from(config_file_data) {
                    Ok(config) => config,
                    Err(_) => {
                        let config = Config::default();
                        let _ = config.save(&config_file_path);
                        config
                    }
                };
                let mapping = get_mapping(&config);
                startup_server(config, mapping).await?;
                Ok(())
            }
            Err(err) => {
                println!("Invalid config file:{path:?}\nError:{error}\nCreating default config", path=&config_file_path, error=err);
                let config = Config::default();
                let _ = config.save(&config_file_path);
                let mapping = get_mapping(&config);
                startup_server(config, mapping).await?;
                Ok(())
            }
        }
    }
    else {
        Ok(())
    }
}

#[cfg(windows)]
fn main() {
    match windows::run() {
        Ok(_) => {} // Service had run
        Err(e) => {
            println!("{:?}", e);
            match e {
                windows::Error::Winapi(winapi_err) => {
                    // If not being run inside windows service framework attempt commandline execution.
                    if winapi_err.raw_os_error().unwrap() == 1063 {
                        let _ = windows::sync_main();
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(not(windows))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    main_service_runtime().await
}