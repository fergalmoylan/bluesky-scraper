mod cid_compat;
mod config;
mod frames;
mod producer;
mod record;

use crate::config::Config;
use anyhow::Result;
use atrium_api::app::bsky::feed::post::Record;
use atrium_api::com::atproto::sync::subscribe_repos::{Commit, NSID};
use atrium_api::types::{CidLink, Collection};
use cid_compat::CidOld;
use frames::Frame;
use futures::StreamExt;
use rdkafka::producer::FutureProducer;
use rdkafka::ClientConfig;
use record::TransformedRecord;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

const FIREHOSE_DOMAIN: &str = "bsky.network";

async fn handle_commit(commit: &Commit, config: &Config, producer: &FutureProducer) -> Result<()> {
    for op in &commit.ops {
        let collection = op.path.split('/').next().expect("op.path is empty");
        if op.action == "create" && collection == atrium_api::app::bsky::feed::Post::NSID {
            let (items, _) = rs_car::car_read_all(&mut commit.blocks.as_slice(), true).await?;
            if let Some((_, item)) = items.iter().find(|(cid, _)| {
                let cid = CidOld::from(*cid)
                    .try_into()
                    .expect("couldn't convert old to new cid");
                Some(CidLink(cid)) == op.cid
            }) {
                let record = serde_ipld_dagcbor::from_reader::<Record, _>(&mut item.as_slice())?;

                let transformed_record = TransformedRecord::from_original(record);
                producer::send_to_kafka(producer, config, transformed_record).await;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env();
    println!("Running with config: {:#?}", &config);

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", config.kafka_addresses.join(","))
        .set("message.timeout.ms", "5000")
        .set("linger.ms", "5")
        .set("batch.size", "16384")
        .set("acks", "1")
        .create()
        .expect("Producer creation error");

    let (mut stream, _) = connect_async(format!("wss://{FIREHOSE_DOMAIN}/xrpc/{NSID}")).await?;
    while let Some(result) = {
        if let Some(Ok(Message::Binary(data))) = stream.next().await {
            Some(Frame::try_from(data.as_slice()))
        } else {
            None
        }
    } {
        if let Ok(Frame::Message(Some(t), message)) = result {
            if t.as_str() == "#commit" {
                let commit = serde_ipld_dagcbor::from_reader(message.body.as_slice())?;
                if let Err(err) = handle_commit(&commit, &config, &producer).await {
                    eprintln!("FAILED: {err:?}");
                }
            }
        }
    }
    Ok(())
}
