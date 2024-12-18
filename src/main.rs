use command::process_commands;

mod command;
mod config;
mod mangalib;
mod processing;
mod rabbitmq_consumer;
mod send_resource;
mod server;

#[tokio::main]
async fn main() -> Result<(), command::Error> {
    config::setup_tracing();
    process_commands().await
}
