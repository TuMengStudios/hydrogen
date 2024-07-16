use axum::Json;

use tracing::debug;
use tracing::info;
use tracing::instrument;

use unique_id::string::StringGenerator;
use unique_id::Generator;

use crate::{
	biz::connector::util,
	core::{AppData, AppErr},
	extractor::RequestContext,
	types::{KafkaCheckRequest, KafkaCheckResponse, KafkaTopicRequest, KafkaTopicResponse},
	util::x_data,
};

pub struct KafkaHandler;

impl KafkaHandler {
	#[tracing::instrument(skip())]
	pub async fn topic_list(
		req_ctx: RequestContext,
		Json(req): Json<KafkaTopicRequest>,
	) -> Result<AppData<KafkaTopicResponse>, AppErr> {
		debug!("fetch kafka topic");
		let res = util::fetch_topic(&req.broker).await;
		x_data(res)
	}

	#[tracing::instrument(skip())]
	pub async fn check(
		req_ctx: RequestContext,
		Json(req): Json<KafkaCheckRequest>,
	) -> Result<AppData<KafkaCheckResponse>, AppErr> {
		debug!("kafka check");
		let res = util::fetch_topic(&req.broker).await;
		x_data(res)
	}
}

impl KafkaHandler {
	#[allow(clippy::default_constructed_unit_structs)]
	#[instrument()]
	pub async fn group_id(req_ctx: RequestContext) -> Result<AppData<String>, AppErr> {
		let id = StringGenerator::default().next_id();
		let group = format!("hydrogen_{id}");
		info!("group {group}");
		let res = Ok(group);
		x_data(res)
	}
}
