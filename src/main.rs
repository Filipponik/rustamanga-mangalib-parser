use command::process_commands;

mod command;
mod config;
mod mangalib;
mod server;
mod send_resource;

#[tokio::main]
async fn main() {
    config::setup_tracing();
    process_commands().await;
}
