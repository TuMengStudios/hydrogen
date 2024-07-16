use std::time::Duration;

use anyhow::Context;

use rdkafka::consumer::BaseConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::ClientConfig;

use tracing::error;

use crate::core::AppErr;
use crate::errcode;

#[tracing::instrument(skip(broker))]
pub async fn fetch_topic(broker: &str) -> Result<Vec<String>, AppErr> {
	// ...
	const OFFSET_TOPICS: &str = "__consumer_offsets";

	// build client
	let consumer_res = ClientConfig::new()
		.set("bootstrap.servers", broker.to_owned())
		.set("session.timeout.ms", "3000")
		.set_log_level(rdkafka::config::RDKafkaLogLevel::Info)
		.create::<BaseConsumer>()
		.with_context(|| format!("create client {broker}"));

	let consumer = match consumer_res {
		Ok(consumer) => consumer,
		Err(err) => {
			error!("build consumer client err {err:?}");
			return Err(errcode::FETCH_TOPIC_CONNECT_ERR.clone());
		}
	};

	// fetch metadata
	let meta = match consumer
		.fetch_metadata(None, Duration::from_secs(30))
		.with_context(|| format!("fetch metadata {broker}"))
	{
		Ok(meta) => meta,
		Err(err) => {
			error!("fetch meta data error {err:?}");
			return Err(errcode::FETCH_TOPIC_METADATA_ERR.clone());
		}
	};

	// filter topic name
	let topics = meta
		.topics()
		.iter()
		// filter offset topics
		.filter(|x| x.name() != OFFSET_TOPICS)
		.map(|x| x.name().to_owned())
		.collect::<Vec<String>>();
	Ok(topics)
}

#[cfg(test)]
mod my_test {
	use crate::biz::connector::util::fetch_topic;

	#[tokio::test]
	async fn test_get_topic() -> anyhow::Result<()> {
		let broker = "localhost:9092";
		let topics = fetch_topic(broker).await?;
		println!("topics {topics:?}");
		Ok(())
	}
}
