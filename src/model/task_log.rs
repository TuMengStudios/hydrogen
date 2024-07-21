use anyhow::Context;

use serde::Deserialize;
use serde::Serialize;

use sqlx::FromRow;
use sqlx::MySql;
use sqlx::MySqlPool;

use tracing::debug;
use tracing::error;
use tracing::info;

use crate::core::AppErr;
use crate::errcode;

#[derive(Debug, Deserialize, FromRow, Serialize)]
pub struct TaskLog {
	pub id: i64,          // task log primary key
	pub task_id: i64,     // about task
	pub log_info: String, // log info
	pub status: i32,      // log status delete or other status
	pub created_at: i64,  // created log time
	pub updated_at: i64,  // updated log time
}

impl TaskLog {
	pub async fn count(conn: &MySqlPool, task_id: i64) -> Result<i64, AppErr> {
		#[derive(FromRow)]
		struct Cnt {
			cnt: i64,
		}

		debug!("count task_log task_id {task_id}");

		let r = sqlx::query_as::<MySql, Cnt>(
			r#"select count(1) as cnt from task_log where task_id = ?"#,
		)
		.bind(task_id)
		.fetch_one(conn)
		.await
		.with_context(|| format!("count task_info where task_id = {}", task_id));

		match r {
			Ok(cnt) => {
				debug!("count task success {} cnt {}", task_id, cnt.cnt);
				Ok(cnt.cnt)
			}
			Err(err) => {
				error!("count task_log error {:?}", err);
				Err(errcode::TASK_LOG_COUNT_ERR.clone())
			}
		}
	}
}

impl TaskLog {
	pub async fn fetch_task_log_list(
		conn: &MySqlPool,
		task_id: i64,
		page: i64,
		page_size: i64,
	) -> Result<Vec<TaskLog>, AppErr> {
		match sqlx::query_as::<MySql, TaskLog>(
			r#"select  * from task_log where task_id = ? limit  ?, ?"#,
		)
		.bind(task_id)
		.bind(page * page_size - page_size)
		.bind(page_size)
		.fetch_all(conn)
		.await
		{
			Ok(res) => Ok(res),
			Err(err) => {
				//
				error!("fetch task log list error {:?}", err);
				Err(errcode::FETCH_TASK_LOG_LIST_ERR.clone())
			}
		}
	}
}
pub enum TaskLogStatus {
	Normal,
	Deleted,
}

impl TaskLogStatus {
	pub fn status(&self) -> i32 {
		match self {
			TaskLogStatus::Normal => 1,
			TaskLogStatus::Deleted => 2,
		}
	}
}

impl TaskLog {
	pub fn new(task_id: i64, log_info: &str) -> Self {
		Self {
			id: 0,
			task_id,
			log_info: log_info.to_string(),
			status: TaskLogStatus::Normal.status(),
			created_at: chrono::Local::now().timestamp(),
			updated_at: chrono::Local::now().timestamp(),
		}
	}
}

impl TaskLog {
	#[tracing::instrument(skip(conn))]
	pub async fn insert_task_log(conn: &MySqlPool, task_log: &mut TaskLog) -> anyhow::Result<()> {
		// todo
		let res = sqlx::query(
			r#"insert into task_log(
			task_id
			,log_info
			,status
			,created_at
			,updated_at) values(
		?
		,?
		,?
		,?
		,?)"#,
		)
		.bind(task_log.task_id)
		.bind(&task_log.log_info)
		.bind(task_log.status)
		.bind(task_log.created_at)
		.bind(task_log.updated_at)
		.execute(conn)
		.await?;

		task_log.id = res.last_insert_id() as i64;
		info!("insert task log success {}", task_log.id);
		Ok(())
	}
}
