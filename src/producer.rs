use crate::app_metrics::KAFKA_LATENCY;
use crate::config::Config;
use crate::record::TransformedRecord;
use log::error;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

pub async fn send_to_kafka(producer: &FutureProducer, config: &Config, payload: TransformedRecord) {
    let kafka_topic = config.kafka_topic.as_str();

    let json_record = tokio::task::spawn_blocking(move || serde_json::to_string(&payload).unwrap())
        .await
        .unwrap();

    //println!("{json_record}\n");

    let produce_future = producer.send(
        FutureRecord::<(), String>::to(kafka_topic).payload(&json_record),
        Duration::from_secs(0),
    );

    let timer = KAFKA_LATENCY.start_timer();
    timer.observe_duration();
    // match produce_future.await {
    //     Ok(..) => {
    //         timer.observe_duration();
    //     }
    //     Err((e, _)) => error!("Error: {:?}", e),
    // }
}
