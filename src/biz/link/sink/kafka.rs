use std::time::Duration;

use anyhow::Context;

use rdkafka::message::OwnedHeaders;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;
use rdkafka::ClientConfig;

use serde::Deserialize;
use serde_json::json;

use tracing::debug;
use tracing::instrument;

use crate::core::CoreMsg;
use crate::util::from_val;

use super::Sinker;

#[derive(Debug, Deserialize)]
struct KafkaSinkArg {
	broker: String,
	topic: String,
}

impl KafkaSinkArg {
	pub fn get_broker(&self) -> &str {
		&self.broker
	}

	pub fn get_topic(&self) -> &str {
		&self.topic
	}
}

#[derive(Debug)]
pub struct KafkaSinker {
	arg: KafkaSinkArg,
	// producer: FutureProducer,
}

impl KafkaSinkArg {
	pub fn new(val: &serde_json::Value) -> anyhow::Result<Self> {
		let r = from_val::<Self>(val)
			.with_context(|| format!("to value error {:?}", serde_json::json!(val).to_string()))?;

		Ok(r)
	}
}

impl KafkaSinker {
	pub fn new(val: &serde_json::Value) -> anyhow::Result<Self> {
		let arg = KafkaSinkArg::new(val)?;
		Ok(Self { arg })
	}
}

impl Sinker for KafkaSinker {
	#[instrument(skip(self, r))]
	async fn sink(
		&self,
		r: tokio::sync::mpsc::Receiver<crate::core::CoreMsg>,
	) -> anyhow::Result<()> {
		self.produce(r).await
	}
}

impl KafkaSinker {
	pub async fn produce(&self, mut r: tokio::sync::mpsc::Receiver<CoreMsg>) -> anyhow::Result<()> {
		let producer = self.producer_client().await?;
		while let Some(msg) = r.recv().await {
			for data in msg.result.iter() {
				let payload = json!(data).to_string();
				debug!("receive message {payload}");
				let key = String::new();
				let headers = OwnedHeaders::new();
				// build record
				let record = FutureRecord::to(self.arg.get_topic())
					.key(&key)
					.payload(&payload)
					.headers(headers);
				// send data
				let _ = producer.send(record, Duration::from_secs(0)).await;
			}
		}
		Ok(())
	}

	#[tracing::instrument(skip(self))]
	async fn producer_client(&self) -> anyhow::Result<FutureProducer> {
		let producer: FutureProducer = ClientConfig::new()
			.set("bootstrap.servers", self.arg.get_broker())
			.set("message.timeout.ms", "5000")
			.set_log_level(rdkafka::config::RDKafkaLogLevel::Info)
			.create()
			.with_context(|| format!("create future producer {}", self.arg.get_broker()))?;
		Ok(producer)
	}
}
