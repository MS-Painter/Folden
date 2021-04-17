use std::{ffi::OsString, time::Duration};

use futures::executor;
use tokio::sync::broadcast;
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

pub async fn sync_main(shutdown_rx: Option<broadcast::Receiver<i32>>) -> Result<(), Box<dyn std::error::Error>> {
    let rt  = Runtime::new()?;
    match shutdown_rx {
        Some(mut rx) => {
            // Exit runtime at service execution termination or if recieved a termination message
            rt.block_on(async {
                tokio::select! {
                    result = super::main_service_runtime() => {
                        match result {
                            Ok(res) => Ok(res),
                            Err(e) => Err(e),
                        }
                    },
                    _ = rx.recv() => Ok(())
                }
            })
        }
        None => {
            // Exit runtime at service execution termination
            rt.block_on(async {
                match  super::main_service_runtime().await {
                    Ok(res) => Ok(res),
                    Err(e) => Err(e),
                }
            })       
        }
    }
}

fn run_service(_arguments: Vec<OsString>) -> Result<(), windows_service::Error> {
    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                // Handle stop event and return control back to the system.
                shutdown_tx.send(1).unwrap();
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

    let sync_main_result = executor::block_on(sync_main(Some(shutdown_rx)));
    let exit_code = match sync_main_result {
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