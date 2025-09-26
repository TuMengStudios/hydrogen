use crate::conf;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::time::ChronoLocal;

// use LogConf to init subscriber
#[allow(clippy::collapsible_else_if)]
pub fn init_log_subscriber(cfg: &conf::LogConfig) -> anyhow::Result<WorkerGuard> {
	let file_appender = tracing_appender::rolling::Builder::new()
		.rotation(tracing_appender::rolling::Rotation::HOURLY)
		.max_log_files(cfg.max_file)
		.filename_prefix(cfg.file_name.as_str())
		.build(cfg.dir.as_str())?;

	let timer = ChronoLocal::new(cfg.time_format.clone());

	let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

	let builder = tracing_subscriber::fmt()
		.with_target(true)
		.with_file(true)
		.with_line_number(true)
		.with_level(true)
		.with_timer(timer)
		.with_max_level(cfg.log_level());

	if cfg.format.to_ascii_lowercase().as_str() == "json" {
		let builder = builder.json();
		if cfg.output.to_ascii_lowercase().as_str() == "console" {
			builder.init();
		} else {
			builder.with_writer(non_blocking).init();
		}
	} else {
		if cfg.output.to_ascii_lowercase().as_str() == "console" {
			builder.init();
		} else {
			builder.with_writer(non_blocking).init();
		}
	}

	Ok(guard)
}
// init log
