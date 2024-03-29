mod config;
mod handler_mapping;
mod mapping;
mod server;
mod startup;

#[cfg(windows)]
use futures::executor::block_on;

#[cfg(windows)]
fn main() {
    match startup::windows::run() {
        Ok(_) => {} // Service finished running
        Err(e) => {
            match e {
                startup::windows::Error::Winapi(winapi_err) => {
                    // If not being run inside windows service framework attempt commandline execution.
                    if winapi_err.raw_os_error().unwrap() == 1063 {
                        tracing::warn!(
                            "Attempting Foldend execution outside of Windows service framework"
                        );
                        block_on(startup::windows::sync_main(None)).unwrap();
                    }
                }
                _ => {
                    tracing::error!("{}", e);
                }
            }
        }
    }
}

#[cfg(not(windows))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    startup::main_service_runtime().await
}
