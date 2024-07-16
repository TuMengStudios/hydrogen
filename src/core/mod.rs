use std::collections::{HashMap, HashSet};

use anyhow::Context;

use axum::response::IntoResponse;
use axum::Json;

use serde::Serialize;

use sqlx::MySqlPool;

use crate::conf;

pub struct ServerContext {
	pub app_state: AppState,
	pub conf: conf::AppConf,
}

impl ServerContext {
	pub async fn new(conf: &conf::AppConf) -> anyhow::Result<Self> {
		let app_state = AppState::new(conf)
			.await
			.with_context(|| format!("init database error {:?}", conf.db))?;
		let data = Self { app_state, conf: conf.clone() };
		Ok(data)
	}
}

#[derive(Clone)]
pub struct AppState {
	pub db_conn: MySqlPool,
}

impl AppState {
	pub async fn new(conf: &conf::AppConf) -> anyhow::Result<Self> {
		// let db = init_db(&conf.db).await?;
		let db_conn = crate::db::init_db_conn(&conf.db).await?;
		let data = Self {
			//db: db,
			db_conn,
		};
		Ok(data)
	}
}

pub struct AppData<T: Serialize>(pub T);

impl<T: Serialize> IntoResponse for AppData<T> {
	fn into_response(self) -> axum::response::Response {
		ResponseData::new(10000, "success".to_owned(), Some(self.0)).into_response()
	}
}

#[derive(Debug, Serialize)]
struct ResponseData<T: Serialize> {
	// err_no
	err_no: i64,
	// err_msg err info
	err_msg: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	// biz body
	data: Option<T>,
}

impl<T: Serialize> ResponseData<T> {
	pub fn new(err_no: i64, err_msg: String, data: Option<T>) -> Self {
		Self { err_msg, err_no, data }
	}
}

impl<T: Serialize> IntoResponse for ResponseData<T> {
	fn into_response(self) -> axum::response::Response {
		Json(self).into_response()
	}
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct AppErr {
	pub err_no: i64,
	pub err_msg: String,
}

impl std::error::Error for AppErr {}
impl std::fmt::Display for AppErr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{self:?}")
	}
}

impl AppErr {
	pub fn new(err_no: i64, err_msg: &str) -> Self {
		Self { err_no, err_msg: err_msg.to_owned() }
	}
}

impl IntoResponse for AppErr {
	fn into_response(self) -> axum::response::Response {
		let data: Option<()> = None;
		ResponseData::new(self.err_no, self.err_msg, data).into_response()
	}
}

#[derive(Debug, Default)]
pub struct CoreMsg {
	pub raw_msg: String,
	pub raw_keys: HashSet<String>,
	pub result: Vec<HashMap<String, serde_json::Value>>,
}

impl CoreMsg {
	pub fn get_raw_msg(&self) -> &str {
		&self.raw_msg
	}
}

impl CoreMsg {
	pub fn with_result(self, result: Vec<HashMap<String, serde_json::Value>>) -> Self {
		Self { result, ..self }
	}

	pub fn with_raw_msg(self, raw_msg: String) -> Self {
		Self { raw_msg, ..self }
	}

	pub fn with_raw_keys(self, row_keys: HashSet<String>) -> Self {
		Self { raw_keys: row_keys, ..self }
	}
}

impl CoreMsg {
	pub fn new(raw_msg: String) -> Self {
		Self { raw_msg, result: vec![], raw_keys: HashSet::new() }
	}
}

unsafe impl Send for CoreMsg {}
