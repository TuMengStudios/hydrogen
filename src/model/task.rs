use std::collections::HashMap;
use std::collections::HashSet;

use anyhow::Context;

use serde::Deserialize;
use serde::Serialize;
use sqlx::prelude::FromRow;
use sqlx::Error;
use sqlx::MySql;
use sqlx::MySqlPool;
use sqlx::Pool;
use sqlx::QueryBuilder;

use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::instrument;

use crate::core::AppErr;
use crate::errcode;
use crate::errcode::DB_INTERNAL_ERROR;

#[derive(Debug, FromRow, Default, Serialize, Clone)]
pub struct TaskInfo {
	pub id: i64,      // id
	pub name: String, // name
	pub status: i32,  // task status
	// pub parser_config: sqlx::types::Json<ParserConfig>, // parser config  json config
	pub parser_config: serde_json::Value, // parser config  json config
	pub src_config: serde_json::Value,    // data source
	pub dst_config: serde_json::Value,    // dst config
	pub debug_text: serde_json::Value,    // debug text
	pub heartbeat: i64,                   // heartbeat
	pub created_at: i64,                  // task created at
	pub updated_at: i64,                  // task updated
	                                      // pub deleted_at: i64,                  // task deleted has value default is 0
}

impl TaskInfo {
	pub fn with_name(self, name: String) -> Self {
		Self { name, ..self }
	}

	pub fn with_src_config(self, src_config: serde_json::Value) -> Self {
		Self { src_config, ..self }
	}

	pub fn with_dst_config(self, dst_config: serde_json::Value) -> Self {
		Self { dst_config, ..self }
	}

	pub fn with_status(self, status: i32) -> Self {
		Self { status, ..self }
	}

	pub fn with_debug_text(self, debug_text: serde_json::Value) -> Self {
		Self { debug_text, ..self }
	}

	pub fn with_parser_config(self, parser_config: serde_json::Value) -> Self {
		Self { parser_config, ..self }
	}

	pub fn with_id(self, id: i64) -> Self {
		Self { id, ..self }
	}

	pub fn get_status(&self) -> i32 {
		self.status
	}
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct ParserConfig {
	pub max_depth: i32,
	pub sep: String,
	pub keys: HashSet<String>,                             // full keys
	pub ignore: HashSet<String>,                           // drop key value
	pub fold: HashSet<String>,                             // fold value
	pub default_value: HashMap<String, serde_json::Value>, // if value is null get  default value
	pub strict_mode: bool,
}

impl TaskInfo {
	pub fn set_id(&mut self, id: i64) {
		self.id = id;
	}

	pub fn set_updated(&mut self, updated_at: i64) {
		self.updated_at = updated_at;
	}

	// pub fn set_created(&self)
}

pub const TASK_STATUS_CREATED: i32 = 1;
pub const TASK_STATUS_DELETED: i32 = -1;
pub const TASK_STATUS_STOP: i32 = 10;
pub const TASK_STATUS_RUNNING: i32 = 12;
pub const TASK_STATUS_ERROR: i32 = 16;

pub enum TaskStatus {
	Created,
	Deleted,
	Running,
	Stop,
	ERROR,
}

impl TaskStatus {
	// status 状态
	pub fn get_status(&self) -> i32 {
		match self {
			TaskStatus::Created => TASK_STATUS_CREATED,
			TaskStatus::Deleted => TASK_STATUS_DELETED,
			TaskStatus::ERROR => TASK_STATUS_ERROR,
			TaskStatus::Running => TASK_STATUS_RUNNING,
			TaskStatus::Stop => TASK_STATUS_STOP,
		}
	}
}

#[derive(Debug)]
pub struct FetchTaskRequest {
	pub status: Option<i32>,
	pub page: i32,
	pub page_size: i32,
}

impl TaskInfo {
	#[tracing::instrument(skip(conn))]
	pub async fn fetch_task_with_status(
		conn: &MySqlPool,
		status: i32,
	) -> Result<Vec<TaskInfo>, AppErr> {
		debug!("fetch task list with status");
		let res = sqlx::query_as::<MySql, TaskInfo>(r#"select * from task_info where status = ?"#)
			.bind(status)
			.fetch_all(conn)
			.await;

		match res {
			Ok(task_list) => {
				//
				debug!("fetch task list success {:?}", task_list);
				Ok(task_list)
			}
			Err(err) => {
				//
				error!("fetch task list error {:?}", err);
				Err(errcode::FETCH_TASK_LIST_ERR.clone())
			}
		}
	}

	// statics status value is {value} count
	#[tracing::instrument(skip(conn))]
	pub async fn get_status_count(conn: &MySqlPool, status: i32) -> Result<i64, AppErr> {
		#[derive(FromRow)]
		struct Cnt {
			cnt: i64,
		}

		debug!("count task status");

		let r = sqlx::query_as::<MySql, Cnt>(
			r#"select count(1) as cnt from task_info where status = ?"#,
		)
		.bind(status)
		.fetch_one(conn)
		.await
		.with_context(|| format!("count task_info where status = {}", status));

		match r {
			Ok(cnt) => {
				debug!("count task success {} cnt {}", status, cnt.cnt);
				Ok(cnt.cnt)
			}
			Err(err) => {
				error!("count task status error {:?}", err);
				Err(errcode::COUNT_TASK_STATUS_ERR.clone())
			}
		}
	}

	#[tracing::instrument(skip(conn))]
	pub async fn get_task(
		conn: &MySqlPool,
		req: FetchTaskRequest,
	) -> Result<Vec<TaskInfo>, AppErr> {
		let mut builder: QueryBuilder<MySql> = QueryBuilder::new("select * from task_info");

		if req.status.is_some() {
			builder.push(" where status = ").push_bind(req.status);
		}
		builder
			.push(" LIMIT ")
			.push_bind(req.page_size)
			.push(" OFFSET ")
			.push_bind(req.page * req.page_size - req.page_size);

		match builder.build_query_as().fetch_all(conn).await {
			Ok(task_list) => {
				info!("fetch task list {:?}", task_list);
				Ok(task_list)
			}
			Err(err) => {
				error!("get task error {:?}", err);
				Err(errcode::FETCH_TASK_LIST_ERR.clone())
			}
		}
	}

	#[tracing::instrument(skip(conn))]
	pub async fn fetch_task_by_id(conn: &Pool<MySql>, id: i64) -> Result<TaskInfo, AppErr> {
		info!("find task by id {}", id);

		let res = sqlx::query_as::<MySql, TaskInfo>(r#"select * from task_info where id = ?"#)
			.bind(id)
			.fetch_one(conn)
			.await;

		match res {
			Ok(tas) => Ok(tas),
			Err(err) => {
				match err {
					Error::RowNotFound => {
						error!("row not found ");
						return Err(errcode::RECORD_NOT_FOUND.clone());
					}
					other => {
						// TODO
						error!("other error {:?}", other);
						return Err(errcode::DB_INTERNAL_ERROR.clone());
					}
				}
			}
		}

		// Ok(res)
	}

	#[instrument(skip(conn))]
	pub async fn create_task(conn: &Pool<MySql>, data: &mut TaskInfo) -> Result<i64, AppErr> {
		debug!("create task");

		let res = sqlx::query(
			r#"insert into task_info(
			name
			,parser_config
			,src_config
			,dst_config
			,debug_text
			,status
			,heartbeat
			,created_at
			,updated_at) values(?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
		)
		.bind(&data.name)
		.bind(&data.parser_config)
		.bind(&data.src_config)
		.bind(&data.dst_config)
		.bind(&data.debug_text)
		.bind(data.status)
		.bind(data.heartbeat)
		.bind(data.created_at)
		.bind(data.updated_at)
		.execute(conn)
		.await;
		//
		match res {
			Ok(res) => {
				data.id = res.last_insert_id() as i64;
			}
			Err(err) => {
				error!("error {:?}", err);
				return Err(DB_INTERNAL_ERROR.clone());
			}
		};
		Ok(data.id)
	}

	#[instrument(skip(conn))]
	pub async fn update_task(conn: &MySqlPool, task: &mut TaskInfo) -> Result<i64, AppErr> {
		debug!("update task id = {}", task.id);
		let res = sqlx::query(
			r#"update task_info set
			status = ?
			, name = ?
			, parser_config = ?
			, src_config = ?
			, dst_config = ?
			, updated_at = ? where id = ?"#,
		)
		.bind(task.status)
		.bind(&task.name)
		.bind(&task.parser_config)
		.bind(&task.src_config)
		.bind(&task.dst_config)
		.bind(task.updated_at)
		.bind(task.id)
		.execute(conn)
		.await;
		//
		match res {
			Ok(res) => {
				debug!("update task success ");
				Ok(res.rows_affected() as i64)
			}
			Err(err) => {
				// err
				error!("update task info id {} error {:?}", task.id, err);
				Err(DB_INTERNAL_ERROR.clone())
			}
		}
	}
}

impl TaskInfo {
	#[tracing::instrument(skip(conn))]
	pub async fn update_heartbeat(conn: &MySqlPool, id: i64) -> anyhow::Result<i64> {
		let heartbeat = chrono::Local::now().timestamp();
		let updated_at = chrono::Local::now().timestamp();

		let res = sqlx::query(
			r#"update task_info set
		 heartbeat = ?
		 ,updated_at = ? where id = ?"#,
		)
		.bind(heartbeat)
		.bind(updated_at)
		.bind(id)
		.execute(conn)
		.await?;

		Ok(res.rows_affected() as i64)
	}
}

#[cfg(test)]
mod test {
	use crate::model::task::TaskStatus;

	use super::TaskInfo;

	// todo
	#[test]
	fn test_build_task() {
		let task = TaskInfo::default()
			.with_id(1)
			.with_status(TaskStatus::Created.get_status())
			.with_name(String::from("foo/baz"));

		assert_eq!(task.id, 1);
		assert_eq!(task.name, String::from("foo/baz"));
	}
}