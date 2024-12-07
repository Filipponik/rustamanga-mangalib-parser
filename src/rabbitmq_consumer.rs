use crate::processing::process;
use futures::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions,
    QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{Connection, ConnectionProperties, ExchangeKind};
use std::env;
use tracing::info;

pub async fn consume(url: &str) {
    let connect = Connection::connect(url, ConnectionProperties::default())
        .await
        .unwrap();
    let channel = connect.create_channel().await.unwrap();

    channel
        .exchange_declare(
            "manga_urls_exchange",
            ExchangeKind::Direct,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let queue = channel
        .queue_declare(
            "manga_urls_queue",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    channel
        .queue_bind(
            queue.name().as_str(),
            "manga_urls_exchange",
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let mut consumer = channel
        .basic_consume(
            queue.name().as_str(),
            "manga_urls_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    info!("Waiting for jobs");
    let chrome_max_count = env::var("CHROME_MAX_COUNT")
        .unwrap()
        .parse::<u16>()
        .unwrap();

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let string_data = std::str::from_utf8(&delivery.data).unwrap();
            info!("Received {}", string_data);
            process(chrome_max_count, serde_json::from_str(string_data).unwrap()).await;

            delivery.ack(BasicAckOptions::default()).await.unwrap();
        }
    }
}
