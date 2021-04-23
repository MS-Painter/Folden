
mod config;
mod server;
mod mapping;
mod startup;

use tempdir;
use tracing_appender;
use tracing_subscriber;
use futures::executor::block_on;

fn setup_tracing() {
    let dir = tempdir::TempDir::new("foldend_logs").expect("Failed to create tempdir");
    let file_appender = tracing_appender::rolling::daily(dir, "foldend.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let collector = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(non_blocking)
        .with_writer(std::io::stdout)
        .finish();
    tracing::subscriber::set_global_default(collector).expect("Unable to set a global collector");
}


#[cfg(windows)]
fn main() {
    setup_tracing();
    match startup::windows::run() {
        Ok(_) => {} // Service finished running
        Err(e) => {
            match e {
                startup::windows::Error::Winapi(winapi_err) => {
                    // If not being run inside windows service framework attempt commandline execution.
                    if winapi_err.raw_os_error().unwrap() == 1063 {
                        tracing::warn!("--- Attempting Foldend execution outside of Windows service framework ---");
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
    setup_tracing();
    startup::main_service_runtime().await
}