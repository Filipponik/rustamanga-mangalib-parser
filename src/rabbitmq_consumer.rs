use crate::processing::{process, ScrapMangaRequest};
use futures::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicQosOptions,
    ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{
    Channel, Connection, ConnectionProperties, Consumer, Error as AmqpError, ExchangeKind, Queue,
};
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
pub enum Error {
    Config(ConfigErrorType),
    Connect(AmqpError),
    ChannelCreate(AmqpError),
    QueueCreate(AmqpError),
    ExchangeCreate(AmqpError),
    QueueBind(AmqpError),
    ConsumerCreate(AmqpError),
    PrefetchSet(AmqpError),
    ParseDelivery(ParseDeliveryErrorType),
    Ack(AmqpError),
    Nack(AmqpError),
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

        let string_data = std::str::from_utf8(&delivery.data)
            .map_err(|err| Error::ParseDelivery(ParseDeliveryErrorType::ParseFromUtf8Error(err)))?;

        info!("Received {}", string_data);
        let payload = serde_json::from_str::<ScrapMangaRequest>(string_data)
            .map_err(|err| Error::ParseDelivery(ParseDeliveryErrorType::ParseJsonError(err)));

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
                    .map_err(Error::Ack)?;
            }
            Err(err) => {
                delivery
                    .nack(BasicNackOptions {
                        requeue: false,
                        ..Default::default()
                    })
                    .await
                    .map_err(Error::Nack)?;
                error!("{err:#?}");
            }
        };
    }

    Ok(())
}

async fn create_channel(url: &str) -> Result<Channel, Error> {
    let connect = Connection::connect(url, ConnectionProperties::default())
        .await
        .map_err(Error::Connect)?;

    connect.create_channel().await.map_err(Error::ChannelCreate)
}

async fn create_queue(channel: &Channel) -> Result<Queue, Error> {
    channel
        .queue_declare(
            "manga_urls_queue",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(Error::QueueCreate)
}

async fn create_exchange(channel: &Channel) -> Result<(), Error> {
    channel
        .exchange_declare(
            "manga_urls_exchange",
            ExchangeKind::Direct,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(Error::ExchangeCreate)
}

async fn queue_bind(channel: &Channel) -> Result<(), Error> {
    channel
        .queue_bind(
            "manga_urls_queue",
            "manga_urls_exchange",
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(Error::QueueBind)
}

async fn create_consumer(channel: &Channel) -> Result<Consumer, Error> {
    channel
        .basic_consume(
            "manga_urls_queue",
            "",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(Error::ConsumerCreate)
}

async fn set_prefetch(channel: &Channel, prefetch_count: u16) -> Result<(), Error> {
    channel
        .basic_qos(prefetch_count, BasicQosOptions::default())
        .await
        .map_err(Error::PrefetchSet)
}

fn get_chrome_max_count() -> Result<u16, Error> {
    env::var("CHROME_MAX_COUNT")
        .map_err(|err| Error::Config(ConfigErrorType::ParseEnv(err)))?
        .parse::<u16>()
        .map_err(|err| Error::Config(ConfigErrorType::ParseInt(err)))
}
