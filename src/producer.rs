use crate::config::Config;
use crate::record::TransformedRecord;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

pub async fn send_to_kafka(producer: &FutureProducer, config: &Config, payload: TransformedRecord) {
    let kafka_topic = config.kafka_topic.as_str();

    let json_record = tokio::task::spawn_blocking(move || serde_json::to_string(&payload).unwrap())
        .await
        .unwrap();

    let produce_future = producer.send(
        FutureRecord::<(), String>::to(kafka_topic).payload(&json_record),
        Duration::from_secs(0),
    );

    match produce_future.await {
        Ok(..) => (),
        Err((e, _)) => eprintln!("Error: {:?}", e),
    }
}
