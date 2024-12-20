use crate::{rabbitmq_consumer, send_resource, server};
use clap::{arg, Command};
use thiserror::Error;

#[allow(clippy::cognitive_complexity)]
fn get_settings() -> Command {
    Command::new("mangalib")
        .about("Mangalib parser")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .version(env!("CARGO_PKG_VERSION"))
        .subcommands([
            Command::new("serve").about("Start web server"),
            Command::new("send-resource")
                .about("Send start static resource")
                .arg(arg!(--url <URL> "URL where we should send this resource"))
                .arg_required_else_help(true),
            Command::new("consume")
                .about("Consume RabbitMQ queue")
                .arg(arg!(--url <URL> "AMQP URI"))
                .arg_required_else_help(true),
        ])
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("No such command {0}")]
    NoSuchCommand(String),
    #[error("No command specified")]
    NoCommandSpecified,
    #[error("Web server error: {0}")]
    Serve(#[from] server::Error),
    #[error("Failed to send resources {0}")]
    SendResource(#[from] send_resource::Error),
    #[error("Failed to consume rabbitmq queue {0}")]
    Consume(#[from] rabbitmq_consumer::Error),
}

pub async fn process_commands() -> Result<(), Error> {
    match get_settings().get_matches().subcommand() {
        Some(("serve", _)) => serve().await,
        Some(("send-resource", sub_matches)) => {
            let url = sub_matches.get_one::<String>("url").expect("required");
            send_resource(url).await
        }
        Some(("consume", sub_matches)) => {
            let url = sub_matches.get_one::<String>("url").expect("required");
            consume(url).await
        }
        Some((command, _)) => Err(Error::NoSuchCommand(command.to_string())),
        None => Err(Error::NoCommandSpecified),
    }
}

async fn serve() -> Result<(), Error> {
    Ok(server::serve().await?)
}

async fn send_resource(url: &str) -> Result<(), Error> {
    Ok(send_resource::send_resource(url).await?)
}

async fn consume(url: &str) -> Result<(), Error> {
    Ok(rabbitmq_consumer::consume(url).await?)
}
