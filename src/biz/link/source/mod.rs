pub mod empty;
pub mod kafka;

use enum_dispatch::enum_dispatch;

use tokio::sync::mpsc;

use kafka::KafkaSource;

use empty::EmptySource;

use crate::core::CoreMsg;

#[enum_dispatch]
pub enum SourceEnum {
	EmptySource,
	KafkaSource,
}

#[allow(async_fn_in_trait)]
#[enum_dispatch(SourceEnum)]
pub trait Source {
	async fn source(&self, s: mpsc::Sender<CoreMsg>) -> anyhow::Result<()>;
}

pub fn get_source(name: &str, val: &serde_json::Value) -> anyhow::Result<SourceEnum> {
	match name.to_lowercase().as_str() {
		"kafka" => Ok(KafkaSource::new(val)?.into()),
		"empty" => Ok(EmptySource::new(val)?.into()),
		other => anyhow::bail!("unknown data source {}", other),
	}
}

#[cfg(test)]
mod my_test {
	use tokio::sync::mpsc;

	use crate::{
		biz::link::{
			sink::{get_sinker, Sinker, SinkerEnum},
			source::{get_source, Source, SourceEnum},
		},
		core::CoreMsg,
	};

	#[tokio::test]
	async fn test_empty() -> anyhow::Result<()> {
		tracing_subscriber::fmt().init();
		let (sender, r) = mpsc::channel::<CoreMsg>(10);
		let source: SourceEnum = get_source("empty", &serde_json::Value::Null)?;
		let sink: SinkerEnum = get_sinker("empty", &serde_json::Value::Null)?;
		let _ = source.source(sender).await;
		let _ = sink.sink(r).await;
		Ok(())
	}
}
