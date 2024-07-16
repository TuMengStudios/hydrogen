use axum::routing::any;
use axum::routing::Router;

use tracing::debug;

use crate::core::AppData;
use crate::core::AppErr;
use crate::extractor::RequestContext;

pub struct HealthHandler;

impl HealthHandler {
	// build router
	pub fn route<S: Clone + Send + Sync + 'static>() -> Router<S> {
		Router::new().route("/health", any(HealthHandler::health))
	}
}

impl HealthHandler {
	#[tracing::instrument(skip())]
	async fn health(req_ctx: RequestContext) -> Result<AppData<i32>, AppErr> {
		debug!("health");
		let res: Result<i32, AppErr> = Ok(1);
		crate::util::x_data(res)
	}
}
