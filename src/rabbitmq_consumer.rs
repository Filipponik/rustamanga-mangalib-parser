use crate::mangalib::http_client::HttpClient;
use crate::processing::Processor;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use lapin::message::Delivery;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicQosOptions,
    ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{
    Channel, Connection, ConnectionProperties, Consumer, Error as AmqpError, ExchangeKind, Queue,
};
use std::env;
use thiserror::Error;
use tokio::time::{Duration, sleep};
use tracing::{error, info};

const QUEUE_NAME: &str = "manga_urls_queue";
const EXCHANGE_NAME: &str = "manga_urls_exchange";

#[derive(Debug, Error)]
pub enum ConfigErrorType {
    #[error("Failed to parse environment variable {0}")]
    ParseEnv(#[from] env::VarError),
    #[error("Failed to parse integer variable {0}")]
    ParseInt(#[from] std::num::ParseIntError),
}

#[derive(Debug, Error)]
pub enum ParseDeliveryErrorType {
    #[error("Failed to parse UTF-8 {0}")]
    ParseFromUtf8Error(#[from] std::str::Utf8Error),
    #[error("Failed to parse json {0}")]
    ParseJsonError(#[from] serde_json::error::Error),
}

#[derive(Debug, Error)]
pub enum AmqpWrapperError {
    #[error("Failed to connect to AMQP {0}")]
    Connect(AmqpError),
    #[error("Failed to create AMQP channel {0}")]
    ChannelCreate(AmqpError),
    #[error("Failed to create AMQP queue {0}")]
    QueueCreate(AmqpError),
    #[error("Failed to create AMQP exchange {0}")]
    ExchangeCreate(AmqpError),
    #[error("Failed to create AMQP consumer {0}")]
    ConsumerCreate(AmqpError),
    #[error("Failed to bind AMQP exchange to queue {0}")]
    QueueBind(AmqpError),
    #[error("Failed to set prefetch AMQP param {0}")]
    PrefetchSet(AmqpError),
    #[error("Failed to ack {0}")]
    Ack(AmqpError),
    #[error("Failed to nack {0}")]
    Nack(AmqpError),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to parse config variable {0}")]
    Config(#[from] ConfigErrorType),
    #[error("AMQP error {0}")]
    Amqp(#[from] AmqpWrapperError),
    #[error("Failed to parse payload {0}")]
    ParseDelivery(#[from] ParseDeliveryErrorType),
    #[error("HTTP client build error {0}")]
    HttpClientBuild(#[from] reqwest::Error),
}

/// Consumes messages from `RabbitMQ` and processes them.
///
/// # Errors
/// Returns an error if connecting, declaring AMQP resources, parsing payloads,
/// or acknowledging/nacking deliveries fails.
pub async fn consume(
    url: &str,
    semaphore_permits: usize,
    proxy_str: Option<&str>,
) -> Result<(), Error> {
    let channel = create_channel(url).await?;
    create_queue(&channel).await?;
    create_exchange(&channel).await?;
    queue_bind(&channel).await?;
    set_prefetch(&channel, 1).await?;

    let mut consumer = create_consumer(&channel).await?;

    info!("Waiting for jobs");
    let client = build_client(proxy_str)?;
    let processor = Processor::new(client, None);
    let mut throttled_until: Option<DateTime<Utc>> = None;

    loop {
        // Check if we're throttled
        if let Some(throttle_time) = throttled_until {
            let now = Utc::now();
            if now < throttle_time {
                #[allow(clippy::cast_sign_loss)]
                let sleep_duration =
                    Duration::from_secs((throttle_time - now).abs().num_seconds() as u64);
                info!("Throttled until {throttle_time:?}, sleeping for {sleep_duration:?}");
                sleep(sleep_duration).await;
            }
        }
        throttled_until = None;

        '_processing_loop: while let Some(Ok(delivery)) = consumer.next().await {
            let payload: Result<&str, ParseDeliveryErrorType> = parse_delivery(&delivery);

            let processing_result = match payload {
                Ok(value) => processor.process(semaphore_permits, value).await,
                Err(err) => {
                    error!("Parse delivery error: {err:?}");
                    continue;
                }
            };

            match processing_result {
                Ok(()) => {
                    delivery
                        .ack(BasicAckOptions::default())
                        .await
                        .map_err(|err| Error::Amqp(AmqpWrapperError::Ack(err)))?;
                }
                Err(crate::processing::Error::Manga(
                    crate::processing::manga::Error::Mangalib(crate::mangalib::Error::Throttling),
                )) => {
                    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
                    let sleep_time = Duration::from_mins(1);
                    let next_start = Utc::now() + sleep_time;
                    throttled_until = Some(next_start);
                    info!("Throttling detected, pausing until {sleep_time:?}");

                    delivery
                        .nack(BasicNackOptions {
                            requeue: true,
                            ..Default::default()
                        })
                        .await
                        .map_err(|err| Error::Amqp(AmqpWrapperError::Nack(err)))?;

                    break '_processing_loop;
                }
                Err(err) => {
                    delivery
                        .nack(BasicNackOptions {
                            requeue: false,
                            ..Default::default()
                        })
                        .await
                        .map_err(|err| Error::Amqp(AmqpWrapperError::Nack(err)))?;
                    error!("Processing error: {err:?}");
                }
            }
        }
    }
}

fn parse_delivery(delivery: &Delivery) -> Result<&str, ParseDeliveryErrorType> {
    let string_data = std::str::from_utf8(&delivery.data)?;
    info!(string_data = string_data, "Received delivery");

    Ok(string_data)
}

async fn create_channel(url: &str) -> Result<Channel, AmqpWrapperError> {
    let connect = Connection::connect(url, ConnectionProperties::default())
        .await
        .map_err(AmqpWrapperError::Connect)?;

    connect
        .create_channel()
        .await
        .map_err(AmqpWrapperError::ChannelCreate)
}

async fn create_queue(channel: &Channel) -> Result<Queue, AmqpWrapperError> {
    channel
        .queue_declare(
            QUEUE_NAME,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(AmqpWrapperError::QueueCreate)
}

async fn create_exchange(channel: &Channel) -> Result<(), AmqpWrapperError> {
    channel
        .exchange_declare(
            EXCHANGE_NAME,
            ExchangeKind::Direct,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(AmqpWrapperError::ExchangeCreate)
}

async fn queue_bind(channel: &Channel) -> Result<(), AmqpWrapperError> {
    channel
        .queue_bind(
            QUEUE_NAME,
            EXCHANGE_NAME,
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(AmqpWrapperError::QueueBind)
}

async fn create_consumer(channel: &Channel) -> Result<Consumer, AmqpWrapperError> {
    channel
        .basic_consume(
            QUEUE_NAME,
            "",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(AmqpWrapperError::ConsumerCreate)
}

async fn set_prefetch(channel: &Channel, prefetch_count: u16) -> Result<(), AmqpWrapperError> {
    channel
        .basic_qos(prefetch_count, BasicQosOptions::default())
        .await
        .map_err(AmqpWrapperError::PrefetchSet)
}

fn build_client(proxy_str: Option<&str>) -> Result<HttpClient, Error> {
    let client_builder = match proxy_str {
        Some(proxy) => {
            let proxy = reqwest::Proxy::all(proxy).map_err(Error::HttpClientBuild)?;
            reqwest::ClientBuilder::new().proxy(proxy)
        }
        None => reqwest::ClientBuilder::new(),
    };

    let client = client_builder.build().map_err(Error::HttpClientBuild)?;

    Ok(HttpClient::builder().reqwest_client(client).build())
}
