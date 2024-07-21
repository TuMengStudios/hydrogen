pub mod connect_handler;
pub mod health;
pub mod kafka_handler;
pub mod metrics;
pub mod parser;
pub mod task_handler;
pub mod task_log_handler;

use tracing::info;

use crate::core::AppData;
use crate::core::AppErr;
use crate::errcode;
use crate::extractor::RequestContext;

#[tracing::instrument(skip())]
pub async fn fallback_handler(req_ctx: RequestContext) -> Result<AppData<()>, AppErr> {
	info!("resource not found");
	let res: Result<_, AppErr> = Err(errcode::NOT_FOUND.clone());
	crate::util::x_data(res)
}
