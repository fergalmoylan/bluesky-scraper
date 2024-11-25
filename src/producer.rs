use crate::config::Config;
use crate::record::TransformedRecord;
use kafka::producer::{Producer, Record, RequiredAcks};
use std::time::Duration;

pub async fn send_to_kafka(config: &Config, payload: TransformedRecord) {
    let kafka_addresses = config.kafka_addresses.clone();
    let kafka_topic = config.kafka_topic.as_str();
    let mut producer = Producer::from_hosts(kafka_addresses)
        .with_ack_timeout(Duration::from_secs(1))
        .with_required_acks(RequiredAcks::One)
        .create()
        .unwrap_or_else(|e| panic!("Failed to create producer: {}", e));

    let json_record = serde_json::to_string(&payload).unwrap();

    let send_result = producer.send(&Record::from_value(kafka_topic, json_record.as_bytes()));

    match send_result {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to send message: {:?}", e),
    }
}
