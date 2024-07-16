use rdkafka::consumer::Consumer;
use rdkafka::consumer::ConsumerContext;
use rdkafka::consumer::Rebalance;
use rdkafka::consumer::StreamConsumer;
use rdkafka::error::KafkaResult;
use rdkafka::ClientConfig;
use rdkafka::ClientContext;
use rdkafka::Message;
use rdkafka::TopicPartitionList;

use serde::Deserialize;
use tokio::sync::mpsc;

use tracing::debug;
use tracing::info;
use tracing::instrument;
use tracing::warn;

use anyhow::Context;

use crate::core::CoreMsg;
use crate::util::from_val;

use super::Source;

#[derive(Debug, Deserialize)]
//pub struct ConsumerArgs {
struct ConsumerArgs {
	broker: String,
	topic: String,
	group_id: String,
}

impl ConsumerArgs {
	pub fn get_broker(&self) -> &str {
		&self.broker
	}

	pub fn get_topic(&self) -> &str {
		&self.topic
	}

	pub fn get_group_id(&self) -> &str {
		&self.group_id
	}
}

struct StreamLoggingCustomContext {}

impl ClientContext for StreamLoggingCustomContext {}

impl ConsumerContext for StreamLoggingCustomContext {
	fn pre_rebalance(&self, rebalance: &Rebalance) {
		warn!("Pre rebalance {:?}", rebalance);
	}

	fn post_rebalance(&self, rebalance: &Rebalance) {
		warn!("Post rebalance {:?}", rebalance);
	}

	fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
		debug!("Committing offsets: {:?} offset {:?}", result, _offsets);
	}
}

type LoggingConsumer = StreamConsumer<StreamLoggingCustomContext>;

pub struct KafkaSource {
	arg: ConsumerArgs,
}

impl KafkaSource {
	pub fn new(conf: &serde_json::Value) -> anyhow::Result<KafkaSource> {
		let arg = from_val(conf)?;
		Ok(Self { arg })
	}
}
impl Source for KafkaSource {
	async fn source(&self, s: mpsc::Sender<CoreMsg>) -> anyhow::Result<()> {
		let consumer = self.streaming_consumer().await?;

		consumer
			.subscribe(&[self.arg.get_topic()])
			.with_context(|| format!("consume topic {}", self.arg.get_topic()))?;
		while let Ok(msg) = consumer.recv().await {
			let offset = msg.offset().abs();
			info!("offset {offset}");
			let res =
				msg.payload_view::<str>().with_context(|| "receive empty message".to_string())?;

			let raw_msg = res
				.with_context(|| {
					format!(
						"covert to str {} topic {} group {}",
						self.arg.get_broker(),
						self.arg.get_topic(),
						self.arg.get_group_id()
					)
				})?
				.to_string();
			let _ = s.send(CoreMsg::default().with_raw_msg(raw_msg)).await;
		}
		Ok(())
	}
}

impl KafkaSource {
	#[instrument(skip(self))]
	async fn streaming_consumer(&self) -> anyhow::Result<LoggingConsumer> {
		let context = StreamLoggingCustomContext {};
		let consumer: LoggingConsumer = ClientConfig::new()
			.set("bootstrap.servers", self.arg.get_broker())
			.set("group.id", self.arg.get_group_id())
			.set("enable.partition.eof", "false")
			.set("session.timeout.ms", "6000")
			.set("enable.auto.commit", "true")
			.set_log_level(rdkafka::config::RDKafkaLogLevel::Info)
			.create_with_context(context)
			.with_context(|| format!("connect broker {}", self.arg.get_broker()))?;

		Ok(consumer)
	}
}
