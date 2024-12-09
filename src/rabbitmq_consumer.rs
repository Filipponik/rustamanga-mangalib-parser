use crate::processing::{process, ScrapMangaRequest};
use futures::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicQosOptions, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{Channel, Connection, ConnectionProperties, Consumer, Error, ExchangeKind, Queue};
use std::env;
use tracing::{error, info};

#[derive(Debug)]
pub enum ConfigErrorType {
    ParseEnv(env::VarError),
    ParseInt(std::num::ParseIntError),
}

#[derive(Debug)]
pub enum ParseDeliveryErrorType {
    ParseFromUtf8Error(std::str::Utf8Error),
    ParseJsonError(serde_json::error::Error),
}

#[derive(Debug)]
pub enum ConsumerError {
    ConfigError(ConfigErrorType),
    ConnectError(Error),
    ChannelCreateError(Error),
    QueueCreateError(Error),
    ExchangeCreateError(Error),
    QueueBindError(Error),
    ConsumerCreateError(Error),
    PrefetchSetError(Error),
    ParseDeliveryError(ParseDeliveryErrorType),
}

pub async fn consume(url: &str) -> Result<(), ConsumerError> {
    let channel = create_channel(url).await?;
    create_queue(&channel).await?;
    create_exchange(&channel).await?;
    queue_bind(&channel).await?;
    set_prefetch(&channel, 1).await?;

    let mut consumer = create_consumer(&channel).await?;

    info!("Waiting for jobs");

    let chrome_max_count = get_chrome_max_count()?;

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let string_data = std::str::from_utf8(&delivery.data)
                .map_err(|err| ConsumerError::ParseDeliveryError(ParseDeliveryErrorType::ParseFromUtf8Error(err)))?;

            info!("Received {}", string_data);
            let payload = serde_json::from_str::<ScrapMangaRequest>(string_data)
                .map_err(|err| ConsumerError::ParseDeliveryError(ParseDeliveryErrorType::ParseJsonError(err)));

            match payload {
                Ok(value) => {
                    process(
                        chrome_max_count,
                        value,
                    )
                        .await;

                    delivery.ack(BasicAckOptions::default()).await.unwrap();
                }
                Err(err) => {
                    error!("{err:#?}");
                    continue;
                }
            }
        }
    }

    Ok(())
}

async fn create_channel(url: &str) -> Result<Channel, ConsumerError> {
    let connect = Connection::connect(url, ConnectionProperties::default())
        .await
        .map_err(ConsumerError::ConnectError)?;

    connect.create_channel().await.map_err(ConsumerError::ChannelCreateError)
}

async fn create_queue(channel: &Channel) -> Result<Queue, ConsumerError> {
    channel
        .queue_declare(
            "manga_urls_queue",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(ConsumerError::QueueCreateError)
}

async fn create_exchange(channel: &Channel) -> Result<(), ConsumerError> {
    channel
        .exchange_declare(
            "manga_urls_exchange",
            ExchangeKind::Direct,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(ConsumerError::ExchangeCreateError)
}

async fn queue_bind(channel: &Channel) -> Result<(), ConsumerError> {
    channel
        .queue_bind(
            "manga_urls_queue",
            "manga_urls_exchange",
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(ConsumerError::QueueBindError)
}

async fn create_consumer(channel: &Channel) -> Result<Consumer, ConsumerError> {
    channel
        .basic_consume(
            "manga_urls_queue",
            "",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(ConsumerError::ConsumerCreateError)
}

async fn set_prefetch(channel: &Channel, prefetch_count: u16) -> Result<(), ConsumerError> {
    channel
        .basic_qos(prefetch_count, BasicQosOptions::default())
        .await
        .map_err(ConsumerError::PrefetchSetError)
}

fn get_chrome_max_count() -> Result<u16, ConsumerError> {
    env::var("CHROME_MAX_COUNT")
        .map_err(|err| ConsumerError::ConfigError(ConfigErrorType::ParseEnv(err)))?
        .parse::<u16>()
        .map_err(|err| ConsumerError::ConfigError(ConfigErrorType::ParseInt(err)))
}