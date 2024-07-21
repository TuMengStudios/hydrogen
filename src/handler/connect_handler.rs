use axum::Json;

use tracing::debug;
use tracing::instrument;

use crate::core::AppData;
use crate::core::AppErr;

use crate::extractor::RequestContext;

use crate::types::KafkaConnectRequest;
use crate::types::KafkaConnectResponse;

pub struct ConnectHandler;

impl ConnectHandler {
	#[instrument]
	pub async fn connect_kafka(
		req_ctx: RequestContext,
		Json(req): Json<KafkaConnectRequest>,
	) -> Result<AppData<KafkaConnectResponse>, AppErr> {
		debug!("this is kafka error {:?}", req_ctx);
		Ok(AppData(()))
	}
}
