use crate::processing::{process, ScrapMangaRequest};
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
}

pub async fn consume(url: &str) -> Result<(), Error> {
    let channel = create_channel(url).await?;
    create_queue(&channel).await?;
    create_exchange(&channel).await?;
    queue_bind(&channel).await?;
    set_prefetch(&channel, 1).await?;

    let mut consumer = create_consumer(&channel).await?;

    info!("Waiting for jobs");

    let chrome_max_count = get_chrome_max_count()?;

    while let Some(delivery) = consumer.next().await {
        let Ok(delivery) = delivery else {
            continue;
        };

        let payload = parse_delivery(&delivery);

        let processing_result = match payload {
            Ok(value) => process(chrome_max_count, value).await,
            Err(err) => {
                error!("{err:#?}");
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
            Err(err) => {
                delivery
                    .nack(BasicNackOptions {
                        requeue: false,
                        ..Default::default()
                    })
                    .await
                    .map_err(|err| Error::Amqp(AmqpWrapperError::Nack(err)))?;
                error!("{err:#?}");
            }
        };
    }

    Ok(())
}

fn parse_delivery_data(data: &[u8]) -> Result<String, ParseDeliveryErrorType> {
    Ok(std::str::from_utf8(data)?.to_string())
}

fn parse_json<T: serde::de::DeserializeOwned>(data: &str) -> Result<T, ParseDeliveryErrorType> {
    Ok(serde_json::from_str::<T>(data)?)
}

fn parse_delivery(delivery: &Delivery) -> Result<ScrapMangaRequest, ParseDeliveryErrorType> {
    let string_data = parse_delivery_data(&delivery.data)?;
    info!("Received {}", string_data);

    parse_json(&string_data)
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

fn get_chrome_max_count() -> Result<u16, ConfigErrorType> {
    Ok(env::var("CHROME_MAX_COUNT")?.parse::<u16>()?)
}
