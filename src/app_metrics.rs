use prometheus::{self, Histogram};
use std::time::SystemTime;

use lazy_static::lazy_static;
use log::info;
use prometheus::core::Number;
use prometheus::register_histogram;

lazy_static! {
    pub static ref KAFKA_LATENCY: Histogram = register_histogram!(
        "records_send_duration_seconds",
        "Time taken to send records to Kafka",
        vec![0.1, 1.0, 5.0]
    )
    .unwrap();
}

pub async fn gather_metrics(start_time: &SystemTime) {
    let counter = KAFKA_LATENCY.get_sample_count().into_f64();
    let timer = KAFKA_LATENCY.get_sample_sum();
    let avg_latency = (timer / counter) * 1000.0;
    let elapsed = start_time.elapsed().unwrap().as_secs();
    let tps = counter / elapsed.into_f64();
    info!(
        "{:<14} {:<10} {:<4} {:<10.1} {:<2} {:<2.2}ms",
        "Total Records:", counter as u64, "TPS:", tps, "Avg Latency:", avg_latency
    );
}
