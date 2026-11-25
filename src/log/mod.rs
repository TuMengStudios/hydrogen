use crate::conf;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;

struct LocalTimer(String);

impl FormatTime for LocalTimer {
	fn format_time(&self, writer: &mut Writer<'_>) -> std::fmt::Result {
		let formatted = jiff::fmt::strtime::format(&self.0, &jiff::Zoned::now())
			.map_err(|_| std::fmt::Error)?;
		writer.write_str(&formatted)
	}
}

// use LogConf to init subscriber
#[allow(clippy::collapsible_else_if)]
pub fn init_log_subscriber(cfg: &conf::LogConfig) -> anyhow::Result<WorkerGuard> {
	let file_appender = tracing_appender::rolling::Builder::new()
		.rotation(tracing_appender::rolling::Rotation::HOURLY)
		.max_log_files(cfg.max_file)
		.filename_prefix(cfg.file_name.as_str())
		.build(cfg.dir.as_str())?;

	let timer = LocalTimer(cfg.time_format.clone());

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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn local_timestamp_preserves_microsecond_format() -> anyhow::Result<()> {
		let timestamp: jiff::Zoned =
			"2026-11-25T01:02:03.123456789+08:00[Asia/Shanghai]".parse()?;
		let formatted = jiff::fmt::strtime::format("%Y-%m-%d %H:%M:%S%.6f", &timestamp)?;
		assert_eq!(formatted, "2026-11-25 01:02:03.123456");
		assert_eq!(timestamp.timestamp().as_second(), 1795539723);
		Ok(())
	}

	#[test]
	fn local_timer_uses_configured_format() -> std::fmt::Result {
		let timer = LocalTimer("literal-%%".to_owned());
		let mut output = String::new();
		timer.format_time(&mut Writer::new(&mut output))?;
		assert_eq!(output, "literal-%");
		Ok(())
	}
}
// init log
