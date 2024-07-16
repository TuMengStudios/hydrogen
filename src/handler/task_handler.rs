use std::time::Duration;

use axum::extract::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;

use sqlx::MySqlPool;

use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::instrument;
use tracing::Instrument;

use crate::biz::job;

use crate::biz::task_manger::contains_task;
use crate::biz::task_manger::remove_task;
use crate::biz::task_manger::running_task;

use crate::core::AppData;
use crate::core::AppErr;
use crate::core::AppState;

use crate::errcode;

use crate::extractor::RequestContext;

use crate::model::task;
use crate::model::task::TaskInfo;
use crate::model::task::TaskStatus;

use crate::types::CreateTaskRequest;
use crate::types::CreateTaskResponse;
use crate::types::FetchTaskRequest;
use crate::types::StartTaskRequest;
use crate::types::StartTaskResponse;
use crate::types::StopTaskRequest;
use crate::types::StopTaskResponse;
use crate::types::TaskCountRequest;
use crate::types::TaskCountResponse;
use crate::types::TaskHealthCheckRequest;
use crate::types::TaskHealthCheckResponse;
use crate::types::TaskListRequest;
use crate::types::TaskListResponse;
use crate::types::UpdateTaskRequest;
use crate::types::UpdateTaskResponse;

use crate::util::x_data;

pub struct TaskHandler;

impl TaskHandler {
	#[tracing::instrument(skip(state))]
	pub async fn fetch_task(
		State(state): State<AppState>,
		req_ctx: RequestContext,
		Path(req): Path<FetchTaskRequest>,
	) -> Result<AppData<TaskInfo>, AppErr> {
		debug!("fetch task by id ");
		let res = TaskInfo::fetch_task_by_id(&state.db_conn, req.id).await;

		info!("fetch task {:?}", res);
		//
		crate::util::x_data(res)
	}
}

impl TaskHandler {
	// create task
	#[tracing::instrument(skip(state))]
	pub async fn create_task(
		// state
		State(state): State<AppState>,
		req_ctx: RequestContext,
		// request body
		Json(req): Json<CreateTaskRequest>,
	) -> Result<AppData<CreateTaskResponse>, AppErr> {
		debug!("create task {:?}", req);

		let mut task = req.to_task();

		let res = TaskInfo::create_task(&state.db_conn, &mut task).await;
		debug!("create task {:?}", res);
		crate::util::x_data(res)
	}
}

impl TaskHandler {
	#[tracing::instrument(skip(state))]
	pub async fn update_task(
		State(state): State<AppState>,
		req_ctx: RequestContext,
		Json(req): Json<UpdateTaskRequest>,
		//
	) -> Result<AppData<UpdateTaskResponse>, AppErr> {
		//
		debug!("update task {:?} uri:{:?}", req, req_ctx.uri);

		// TaskInfo::default();
		let mut task = req.cover_task();

		let r = TaskInfo::update_task(&state.db_conn, &mut task).await;
		// Ok(AppData(r))
		crate::util::x_data(r)
	}
}

impl TaskHandler {
	#[tracing::instrument(skip(state))]
	pub async fn start_task(
		State(state): State<AppState>,
		Path(req): Path<StartTaskRequest>,
	) -> Result<AppData<StartTaskResponse>, AppErr> {
		// todo
		debug!("request req  {:?}", req);
		if contains_task(req.id) {
			error!("task has running {:?}", req.id);
			return Err(errcode::TASKING_ALREADY_RUNNING.clone());
		}
		let task = TaskInfo::fetch_task_by_id(&state.db_conn, req.id).await?;
		let res = job::Tasking::start_task(task, state.db_conn).await;
		x_data(res)
	}
}

impl TaskHandler {
	#[tracing::instrument(skip(state, req))]
	pub async fn stop_task(
		State(state): State<AppState>,
		Path(req): Path<StopTaskRequest>,
	) -> Result<AppData<StopTaskResponse>, AppErr> {
		// todo
		debug!("stop task req {:?}", req);
		let task = TaskInfo::fetch_task_by_id(&state.db_conn, req.id)
			.instrument(tracing::Span::current())
			.await?;

		// task status is not running
		if task.status != TaskStatus::Running.get_status() {
			info!("task {} not running", req.id);
			return Err(errcode::TASK_NOT_RUNNING.clone());
		}

		// task not in task manger
		if !contains_task(req.id) {
			info!("not found task {} in task manger ", req.id);
			return Err(errcode::TASK_NOT_RUNNING.clone());
		}
		//
		info!("stop task {task:?}");
		// remove task from task manger and then trigger handle.cancel()
		remove_task(req.id);
		Ok(AppData(()))
	}
}

impl TaskHandler {
	// count task status
	#[tracing::instrument(skip(state, req_ctx))]
	pub async fn task_count(
		State(state): State<AppState>,
		req_ctx: RequestContext,
		Query(req): Query<TaskCountRequest>,
	) -> Result<AppData<TaskCountResponse>, AppErr> {
		//
		debug!("task count {:?} uri:{:?} ", req, req_ctx.uri);
		// parser status default is 0
		let status = req.status.unwrap_or(0);

		// count task status from db
		let res = TaskInfo::get_status_count(&state.db_conn, status).await;
		crate::util::x_data(res)
	}
}

impl TaskHandler {
	#[tracing::instrument(skip(state))]
	pub async fn list_task(
		State(state): State<AppState>,
		req_ctx: RequestContext,
		Query(req): Query<TaskListRequest>,
	) -> Result<AppData<TaskListResponse>, AppErr> {
		debug!("task list");
		// build request arg
		let page_arg = task::FetchTaskRequest {
			status: Some(req.status),
			page: req.page,
			page_size: req.page_size,
		};
		// ...
		debug!("list task page arg {:?}", page_arg);
		//  get task list from db
		let res = TaskInfo::get_task(&state.db_conn, page_arg).await;
		// response result
		crate::util::x_data(res)
	}
}

impl TaskHandler {
	pub async fn running_task() -> Result<AppData<Vec<i64>>, AppErr> {
		Ok(AppData(running_task()))
	}
}

impl TaskHandler {
	#[tracing::instrument(skip(state))]
	pub async fn health(
		req_ctx: RequestContext,
		State(state): State<AppState>,
		Json(req): Json<TaskHealthCheckRequest>,
	) -> Result<AppData<TaskHealthCheckResponse>, AppErr> {
		if !contains_task(req.get_id()) {
			error!("task is not running {}", req.get_id());
			return Err(errcode::TASK_HEALTH_ERR.clone());
		}

		let task = TaskInfo::fetch_task_by_id(&state.db_conn, req.get_id()).await?;
		if task.get_status() != TaskStatus::Running.get_status() {
			error!("task {} is not running {}", task.id, task.get_status());
		}

		return Ok(AppData(task.get_status()));
	}
}

impl TaskHandler {
	// while app exit the status running
	pub async fn reload_running_job(conn: MySqlPool) {
		let task_res =
			TaskInfo::fetch_task_with_status(&conn, TaskStatus::Running.get_status()).await;
		let tasks = match task_res {
			Ok(tasks) => tasks,
			Err(err) => {
				error!("fetch running task error {:?}", err);
				return;
			}
		};

		for task in tasks {
			info!("restart task {}", task.id);
			if contains_task(task.id) {
				//
				info!("task is running {:?}", task.id);
				continue;
			}

			let res = job::Tasking::start_task(task, conn.clone()).await;
			if res.is_err() {
				error!("start task error {:?}", res.err());
			} else {
				info!("restart task success");
			}
		}
	}
}

#[instrument]
pub async fn long_time_task() -> Result<i64, AppErr> {
	error!("long time task");
	tokio::time::sleep(Duration::from_secs(4)).await;
	Err(errcode::DB_INTERNAL_ERROR.clone())
}

#[instrument]
pub async fn long_time_task2() -> Result<i64, AppErr> {
	info!("long time task2");
	tokio::time::sleep(Duration::from_secs(10)).await;
	Err(errcode::DB_INTERNAL_ERROR.clone())
}
#[cfg(test)]
mod task_manager_test {
	// todo
}
