use futures::StreamExt;
use lapin::{Connection, ConnectionProperties, ExchangeKind};
use lapin::options::{BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions};
use lapin::types::FieldTable;

pub async fn consume(url: &str) {
    let connect = Connection::connect(url, ConnectionProperties::default()).await.unwrap();
    let channel = connect.create_channel().await.unwrap();

    let _exchange = channel
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

    channel.queue_bind(
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

    println!("Waiting for jobs");

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            println!("Received {:?}", std::str::from_utf8(&delivery.data).unwrap_or("WTF I CANT DECODE!"));
        }
    }
}