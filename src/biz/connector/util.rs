use std::time::Duration;

use anyhow::Context;

use query_map::QueryMap;
use rdkafka::consumer::BaseConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::ClientConfig;

use tracing::debug;
use tracing::error;

use crate::core::AppErr;
use crate::errcode;

#[tracing::instrument(skip(params))]
pub async fn fetch_topic(params: &str) -> Result<Vec<String>, AppErr> {
	// ...

	const OFFSET_TOPICS: &str = "__consumer_offsets";
	let connect_map = match params
		.parse::<QueryMap>()
		.with_context(|| format!("parser connect url error {}", params))
	{
		Ok(data) => data,
		Err(err) => {
			error!("parser connect url error {:?}", err);
			return Err(errcode::FETCH_TOPIC_CONNECT_ERR.clone());
		}
	};
	let mut config = ClientConfig::new();
	for (k, v) in connect_map.iter() {
		config.set(k, v);
		debug!("set config {}={}", k, v);
	}
	let consumer_res = config
		.set_log_level(rdkafka::config::RDKafkaLogLevel::Info)
		.create::<BaseConsumer>()
		.with_context(|| format!("create client {params}"));

	let consumer = match consumer_res {
		Ok(consumer) => consumer,
		Err(err) => {
			error!("build consumer client err {err:?}");
			return Err(errcode::FETCH_TOPIC_PROPERTY_ERR
				.clone()
				.with_err_msg(format!("{:?}", err)));
		}
	};

	// fetch metadata
	let meta = match consumer
		.fetch_metadata(None, Duration::from_secs(30))
		.with_context(|| format!("fetch metadata {params}"))
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
		let params = "bootstrap.servers=localhost:9092";
		let topics = fetch_topic(params).await?;
		println!("topics {topics:?}");
		Ok(())
	}
}
