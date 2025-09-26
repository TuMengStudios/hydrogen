use serde::Deserialize;
use serde::Serialize;

use tracing::debug;
use tracing::Level;

#[derive(Deserialize, Debug, Clone)]
pub struct AppConf {
	pub name: String,
	pub id: i32,
	pub http: HttpServer,
	pub db: DBConf,
	pub log: LogConfig,
}

impl AppConf {
	#[tracing::instrument]
	pub fn from_path(path: &str) -> anyhow::Result<Self> {
		let content = match std::fs::read_to_string(path) {
			Ok(content) => content,
			Err(err) => {
				anyhow::bail!("open file {} error {:?}", path, err);
			}
		};
		debug!("read content {}", content);
		let app = match toml::from_str(&content) {
			Ok(app) => app,
			Err(err) => {
				anyhow::bail!("parser file error {:?}", err)
			}
		};
		debug!("app config {:?}", app);
		anyhow::Ok(app)
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct HttpServer {
	pub endpoint: String,
}

impl HttpServer {
	#[tracing::instrument]
	pub fn get_endpoint(&self) -> &str {
		&self.endpoint
	}
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct DBConf {
	pub dsn: String,
	pub max_conn: u32,
}

impl DBConf {
	pub fn get_dsn(&self) -> &str {
		&self.dsn
	}
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
	pub file_name: String,
	pub dir: String,
	pub max_file: usize,
	pub time_format: String, // time format,default %Y-%m-%d %H:%M:%S%.6f
	pub format: String,
	pub level: String, // log level  trace,debug,info,warn,error
	pub output: String,
}

impl LogConfig {
	pub fn log_level(&self) -> Level {
		println!("get log level {}", self.level.to_ascii_lowercase().as_str());
		match self.level.to_ascii_lowercase().as_str() {
			"trace" => Level::TRACE,
			"debug" => Level::DEBUG,
			"info" => Level::INFO,
			"warn" => Level::WARN,
			"error" => Level::ERROR,
			other => {
				// have not register log subscriber yet so use stdout
				println!("unknown log level {}", other);
				Level::INFO
			}
		}
	}
}
