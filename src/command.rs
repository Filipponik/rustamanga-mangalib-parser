use crate::{collect_resource, config, rabbitmq_consumer, send_resource, server};
use clap::{ArgMatches, Command, arg};
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
            Command::new("serve")
                .about("Start web server")
                .arg(arg!(--port <PORT> "Web server port"))
                .arg(arg!(--semaphore_permits <SEMAPHORE_PERMITS> "Max semaphore permits")),
            Command::new("send-resource")
                .about("Send start static resource")
                .arg(arg!(--url <URL> "URL where we should send this resource"))
                .arg_required_else_help(true),
            Command::new("collect-resource-full").about("Collect current resource to json"),
            Command::new("consume")
                .about("Consume RabbitMQ queue")
                .arg(arg!(--url <URL> "AMQP URI"))
                .arg(arg!(--proxy <PROXY> "Proxy URI"))
                .arg(arg!(--semaphore_permits <SEMAPHORE_PERMITS> "Max semaphore permits"))
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
    #[error("Failed to parse arguments: {0}")]
    BadArgument(String),
    #[error("Failed to collect resource: {0}")]
    CollectResource(#[from] collect_resource::Error),
}

/// Processes CLI commands and dispatches handlers.
///
/// # Errors
/// Returns an error if an unknown command is supplied, argument parsing fails, or downstream handlers fail.
pub async fn process_commands() -> Result<(), Error> {
    match get_settings().get_matches().subcommand() {
        Some(("serve", sub_matches)) => {
            let port = parse_port(sub_matches)?;
            let semaphore_permits = parse_semaphore_permits(sub_matches)?;

            serve(port, semaphore_permits).await
        }
        Some(("send-resource", sub_matches)) => {
            let url = sub_matches
                .get_one::<String>("url")
                .ok_or_else(|| Error::BadArgument("url is required".to_string()))?;
            send_resource(url).await
        }
        Some(("consume", sub_matches)) => {
            let url = sub_matches
                .get_one::<String>("url")
                .ok_or_else(|| Error::BadArgument("url is required".to_string()))?;
            let proxy_str = sub_matches.get_one::<String>("proxy").map(String::as_str);
            let semaphore_permits = parse_semaphore_permits(sub_matches)?;

            consume(url, semaphore_permits, proxy_str).await
        }
        Some(("collect-resource-full", _sub_matches)) => {
            Ok(collect_resource::collect_resource().await?)
        }
        Some((command, _)) => Err(Error::NoSuchCommand(command.to_string())),
        None => Err(Error::NoCommandSpecified),
    }
}

fn parse_semaphore_permits(sub_matches: &ArgMatches) -> Result<usize, Error> {
    sub_matches
        .get_one::<String>("semaphore_permits")
        .map_or_else(
            || Ok(config::DEFAULT_SEMAPHORE_PERMITS),
            |value| {
                value.parse::<usize>().map_err(|err| {
                    Error::BadArgument(format!("Failed to parse semaphore permits: {err}"))
                })
            },
        )
}

fn parse_port(sub_matches: &ArgMatches) -> Result<u16, Error> {
    sub_matches.get_one::<String>("port").map_or_else(
        || Ok(config::DEFAULT_APP_PORT),
        |value| {
            value
                .parse::<u16>()
                .map_err(|err| Error::BadArgument(format!("Failed to parse port: {err}")))
        },
    )
}

async fn serve(port: u16, semaphore_permits: usize) -> Result<(), Error> {
    Ok(server::serve(port, semaphore_permits).await?)
}

async fn send_resource(url: &str) -> Result<(), Error> {
    Ok(send_resource::send_resource(url).await?)
}

async fn consume(
    url: &str,
    semaphore_permits: usize,
    proxy_str: Option<&str>,
) -> Result<(), Error> {
    Ok(rabbitmq_consumer::consume(url, semaphore_permits, proxy_str).await?)
}
