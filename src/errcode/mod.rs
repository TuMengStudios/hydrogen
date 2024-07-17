use lazy_static::lazy_static;

use crate::core::AppErr;

// SUCCESS
lazy_static! {
	// success
	pub static ref SUCCESS: AppErr = AppErr::new(10000, "success");
}

lazy_static! {
	// method not found
	pub static ref NOT_FOUND: AppErr = AppErr::new(404, "resources not found");
}

// PARSER_ERROR
lazy_static! {
	pub static ref PARSER_ERROR: AppErr = AppErr::new(20000, "parser json property error");
}

// database error
lazy_static! {
	// not found record
	pub static ref RECORD_NOT_FOUND: AppErr = AppErr::new(30000, "record not found");
	// db error
	pub static ref DB_INTERNAL_ERROR: AppErr = AppErr::new(30001, "internal error");
}

lazy_static! {
	// task not running
	pub static ref TASK_NOT_RUNNING: AppErr = AppErr::new(30000, "task not running");
	// task is running
	pub static ref TASK_IS_RUNNING: AppErr = AppErr::new(30001,"task is running");
	// task not found
	pub static ref TASK_NOT_FOUND:AppErr = AppErr ::new(30002,"task not found");
}

lazy_static! {
	pub static ref COUNT_TASK_STATUS_ERR: AppErr = AppErr::new(30100, "count task status");
}

lazy_static! {
	pub static ref FETCH_TASK_LIST_ERR: AppErr = AppErr::new(30200, "fetch task list error");
}

lazy_static! {
	pub static ref FETCH_TOPIC_ERR: AppErr = AppErr::new(30300, "fetch topic error");
	pub static ref FETCH_TOPIC_CONNECT_ERR: AppErr = AppErr::new(30301, "connect error");
	pub static ref FETCH_TOPIC_METADATA_ERR: AppErr = AppErr::new(30302, "fetch metadata error");
	pub static ref FETCH_TOPIC_PROPERTY_ERR: AppErr = AppErr::new(30303, "property error");
}

lazy_static! {
	pub static ref TASKING_START_ERR: AppErr = AppErr::new(30400, "build tasking");
	pub static ref TASKING_ALREADY_RUNNING: AppErr = AppErr::new(30401, "task already running");
}

lazy_static! {
	pub static ref TASK_HEALTH_ERR: AppErr = AppErr::new(30500, "task not running");
}

lazy_static! {
	pub static ref SYSTEM_MONITOR_ERR: AppErr = AppErr::new(30600, "system monitor error");
}

lazy_static! {
	pub static ref TASK_LOG_COUNT_ERR: AppErr = AppErr::new(30700, "task log count error");
	pub static ref FETCH_TASK_LOG_LIST_ERR: AppErr = AppErr::new(30701, "task log list error");
}

lazy_static! {
	pub static ref FETCH_METRICS_ERR: AppErr = AppErr::new(30800, "fetch metrics error");
}
