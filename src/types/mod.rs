use lepumk::ani;
use serde::Deserialize;
use serde::Serialize;

use serde_json::json;
use tokio_metrics::TaskMetrics;

use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;

use tracing::debug;

use crate::model::task::TaskInfo;
use crate::model::task::TaskStatus;
use crate::model::task_log::TaskLog;

#[derive(Debug, Deserialize, Serialize)]
pub struct PropertyPlainTextRequest {
	pub plain_text: serde_json::Value,
	pub sep: Option<String>,
}

impl PropertyPlainTextRequest {
	pub fn to_str(&self) -> String {
		json!(self.plain_text).to_string()
	}
}

impl PropertyPlainTextRequest {
	pub fn get_sep(&self) -> &str {
		match &self.sep {
			Some(sep) => {
				debug!("get user define sep {}", sep);
				sep
			}
			None => {
				debug!("use system sep _");
				"_"
			}
		}
	}
}

impl PropertyPlainTextRequest {
	pub fn to_json_parser(&self) -> ani::JsonParser {
		ani::ParserOptions::fmt().with_sep(self.get_sep()).init()
	}
}

// use property as response
pub type PropertyPlainTextResponse = lepumk::ani::Property;

#[derive(Debug, Deserialize)]
pub struct ParserPlainTextRequest {
	pub max_depth: i32,                                    // default 0 yep ignore this
	pub sep: String,                                       // {pre_key}{sep}{current_key}
	pub keys: HashSet<String>,                             // full keys
	pub ignore: HashSet<String>,                           // drop key value
	pub fold: HashSet<String>,                             // fold value
	pub default_value: HashMap<String, serde_json::Value>, // if value is null get  default value
	pub strict_mode: bool,                                 // run in strict mode or not
	pub debug_text: serde_json::Value,                     // demo text
}

impl ParserPlainTextRequest {
	pub fn debug_str(&self) -> String {
		json!(self.debug_text).to_string()
	}
}

impl ParserPlainTextRequest {
	pub fn get_sep(&self) -> &str {
		&self.sep
	}

	pub fn get_fold(&self) -> HashSet<String> {
		self.fold.clone()
	}

	pub fn get_keys(&self) -> HashSet<String> {
		self.keys.clone()
	}

	pub fn get_ignore(&self) -> HashSet<String> {
		self.ignore.clone()
	}
}

impl ParserPlainTextRequest {
	pub fn to_parser_json_parser(&self) -> ani::JsonParser {
		ani::ParserOptions::fmt()
			.with_sep(self.get_sep())
			.with_strict_mode(self.strict_mode)
			.with_fold(self.get_fold())
			.with_ignore(self.get_ignore())
			.with_max_depth(self.max_depth)
			.with_keys(self.get_keys())
			.with_default_value(self.default_value.clone())
			.init()
	}
}

// parser plain text response is a vector that contain every parser item
pub type ParserPlainTextResponse = Vec<HashMap<String, serde_json::Value>>;

#[derive(Deserialize, Debug, Serialize)]
pub struct FetchTaskRequest {
	pub id: i64,
}

pub type FetchTaskResponse = crate::model::task::TaskInfo;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTaskRequest {
	pub name: String,
	pub parser_config: crate::model::task::ParserConfig,
	pub debug_text: serde_json::Value,
	pub dst_config: serde_json::Value,
	pub src_config: serde_json::Value,
	pub property_item: PropertyItem, //
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PropertyItem {
	pub node_name: String,
	pub value_type: String,
	pub op: String,
	pub props: Vec<PropertyItem>,
}

#[derive(Debug, Deserialize)]
pub struct PropertyItemRequest {
	pub node_name: String,  // node or sub node name
	pub value_type: String, // value type
	pub op: String,
	pub props: Vec<PropertyItemRequest>,
}

impl CreateTaskRequest {
	pub fn to_task(&self) -> TaskInfo {
		let mut data = TaskInfo::default();
		data.name.clone_from(&self.name);
		data.created_at = chrono::Local::now().timestamp();
		data.updated_at = chrono::Local::now().timestamp();
		data.parser_config = serde_json::json!(self.parser_config.clone());
		// data.parser_config = self.parser_config.clone();
		data.debug_text = self.debug_text.clone();
		data.dst_config = self.dst_config.clone();
		data.src_config = self.src_config.clone();
		data.status = TaskStatus::Created.get_status();
		data
	}
}

#[derive(Debug, Deserialize)]
pub struct KafkaSrcConfig {
	pub broker: String,
	pub topic: String,
}

#[derive(Debug, Deserialize)]
pub struct KafkaDstConfig {
	pub broker: String,
}

#[derive(Debug, Deserialize)]
pub struct KafkaTopicRequest {
	pub params: String,
}

pub type KafkaTopicResponse = Vec<String>;

pub type KafkaCheckRequest = KafkaTopicRequest;
pub type KafkaCheckResponse = KafkaTopicResponse;

pub type CreateTaskResponse = i64;

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateTaskRequest {
	pub id: i64,
	pub name: String,
	pub status: i32,
	//pub parser_config: serde_json::Value,
	pub parser_config: crate::model::task::ParserConfig,
	pub dst_config: serde_json::Value,
	pub src_config: serde_json::Value,
	pub debug_text: serde_json::Value,
	pub property_item: PropertyItem, //
}

impl UpdateTaskRequest {
	pub fn cover_task(&self) -> TaskInfo {
		TaskInfo::default()
			.with_id(self.id)
			.with_name(self.name.clone())
			.with_status(self.status)
			.with_parser_config(serde_json::to_value(self.parser_config.clone()).unwrap())
			.with_src_config(self.src_config.clone())
			.with_dst_config(self.dst_config.clone())
			.with_debug_text(self.debug_text.clone())
	}
}

pub type UpdateTaskResponse = i64;

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskCountRequest {
	pub status: Option<i32>,
}

pub type TaskCountResponse = i64;

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskListRequest {
	pub status: i32,
	pub page_size: i32,
	pub page: i32,
}

pub type TaskListResponse = Vec<TaskInfo>;

#[derive(Deserialize, Debug, Serialize)]
pub struct KafkaConnectRequest {
	pub brokers: Vec<String>,
	pub topic: Option<String>,
}

pub type KafkaConnectResponse = ();

// json parser config
#[derive(Debug, Deserialize)]
pub struct JsonParserOpt {
	pub max_depth: i32,                                    // default 0 yep ignore this
	pub sep: String,                                       // {pre_key}{sep}{current_key}
	pub keys: HashSet<String>,                             // full keys
	pub ignore: HashSet<String>,                           // drop key value
	pub fold: HashSet<String>,                             // fold value
	pub default_value: HashMap<String, serde_json::Value>, // if value is null get  default value
	pub strict_mode: bool,                                 // run in strict mode or not
}

impl JsonParserOpt {
	pub fn to_parser(&self) -> ani::JsonParser {
		ani::ParserOptions::fmt()
			.with_sep(self.get_sep())
			.with_strict_mode(self.strict_mode)
			.with_fold(self.get_fold())
			.with_ignore(self.get_ignore())
			.with_max_depth(self.max_depth)
			.with_keys(self.get_keys())
			.with_default_value(self.default_value.clone())
			.init()
	}
}

impl JsonParserOpt {
	pub fn get_sep(&self) -> &str {
		&self.sep
	}

	pub fn get_fold(&self) -> HashSet<String> {
		self.fold.clone()
	}

	pub fn get_keys(&self) -> HashSet<String> {
		self.keys.clone()
	}

	pub fn get_ignore(&self) -> HashSet<String> {
		self.ignore.clone()
	}
}
#[derive(Debug, Deserialize)]
pub struct StartTaskRequest {
	pub id: i64,
}

pub type StartTaskResponse = ();

#[derive(Debug, Deserialize)]
pub struct StopTaskRequest {
	pub id: i64,
}

pub type StopTaskResponse = ();

#[derive(Debug, Deserialize)]
pub struct TaskHealthCheckRequest {
	pub id: i64,
}

impl TaskHealthCheckRequest {
	pub fn get_id(&self) -> i64 {
		self.id
	}
}

pub type TaskHealthCheckResponse = i32;

#[derive(Debug, Deserialize)]
pub struct SystemMonitorRequest {}

#[derive(Debug, Serialize, Default)]
pub struct SystemMonitorResponse {
	pub cpu_usage: f64,
	pub memory_usage: f64,
}

impl SystemMonitorResponse {
	pub fn with_cpu_usage(self, cpu_usage: f64) -> Self {
		Self { cpu_usage, ..self }
	}

	pub fn with_memory_usage(self, memory_usage: f64) -> Self {
		Self { memory_usage, ..self }
	}
}

#[derive(Debug, Deserialize)]
pub struct FetchTaskLogListRequest {
	pub task_id: i64,
	pub page: i64,
	pub page_size: i64,
}

pub type FetchTaskLogListResponse = Vec<TaskLog>;

#[derive(Debug, Deserialize)]
pub struct TaskLogCountRequest {
	pub task_id: i64,
}

pub type TaskLogCountResponse = i64;

#[derive(Serialize, Debug)]
pub struct TokioMetrics {
	pub instrumented_count: u64,
	pub dropped_count: u64,
	pub first_poll_count: u64,
	pub total_first_poll_delay: Duration,
	pub total_idled_count: u64,
	pub total_idle_duration: Duration,
	pub total_scheduled_count: u64,
	pub total_scheduled_duration: Duration,
	pub total_poll_count: u64,
	pub total_poll_duration: Duration,
	pub total_fast_poll_count: u64,
	pub total_fast_poll_duration: Duration,
	pub total_slow_poll_count: u64,
	pub total_slow_poll_duration: Duration,
	pub total_short_delay_count: u64,
	pub total_long_delay_count: u64,
	pub total_short_delay_duration: Duration,
	pub total_long_delay_duration: Duration,
}

impl TokioMetrics {
	pub fn from_tokio_task_metrics(m: TaskMetrics) -> Self {
		Self {
			instrumented_count: m.instrumented_count,
			dropped_count: m.dropped_count,
			first_poll_count: m.first_poll_count,
			total_fast_poll_count: m.total_fast_poll_count,
			total_idle_duration: m.total_idle_duration,
			total_first_poll_delay: m.total_first_poll_delay,
			total_idled_count: m.total_idled_count,
			total_scheduled_count: m.total_scheduled_count,
			total_scheduled_duration: m.total_scheduled_duration,
			total_poll_count: m.total_poll_count,
			total_fast_poll_duration: m.total_fast_poll_duration,
			total_poll_duration: m.total_poll_duration,
			total_slow_poll_count: m.total_slow_poll_count,
			total_slow_poll_duration: m.total_slow_poll_duration,
			total_short_delay_count: m.total_short_delay_count,
			total_long_delay_count: m.total_long_delay_count,
			total_short_delay_duration: m.total_short_delay_duration,
			total_long_delay_duration: m.total_long_delay_duration,
		}
	}
}
