use command::process_commands;
use tracing::error;

mod command;
mod config;
mod mangalib;
mod processing;
mod rabbitmq_consumer;
mod send_resource;
mod server;

#[tokio::main]
async fn main() -> Result<(), command::Error> {
    config::setup_logging_console_text();
    if let Err(err) = process_commands().await {
        error!("Error: {}", err);
    }

    Ok(())
}
