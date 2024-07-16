use axum::extract::Query;
use axum::extract::State;

use crate::util::x_data;

use crate::core::AppData;
use crate::core::AppErr;
use crate::core::AppState;

use crate::extractor::RequestContext;

use crate::types::FetchTaskLogListRequest;
use crate::types::FetchTaskLogListResponse;

use crate::types::TaskLogCountRequest;
use crate::types::TaskLogCountResponse;

use crate::model::task_log::TaskLog;
pub struct TaskLogHandler {}

impl TaskLogHandler {
	#[tracing::instrument(skip(state))]
	pub async fn task_log_list(
		State(state): State<AppState>,
		req_ctx: RequestContext,
		Query(req): Query<FetchTaskLogListRequest>,
	) -> Result<AppData<FetchTaskLogListResponse>, AppErr> {
		let res =
			TaskLog::fetch_task_log_list(&state.db_conn, req.task_id, req.page, req.page_size)
				.await;
		x_data(res)
	}
}

impl TaskLogHandler {
	#[tracing::instrument(skip(state))]
	pub async fn task_log_count(
		State(state): State<AppState>,
		req_ctx: RequestContext,
		Query(req): Query<TaskLogCountRequest>,
	) -> Result<AppData<TaskLogCountResponse>, AppErr> {
		let res = TaskLog::count(&state.db_conn, req.task_id).await;
		x_data(res)
	}
}
