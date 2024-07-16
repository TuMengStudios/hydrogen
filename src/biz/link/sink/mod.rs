use enum_dispatch::enum_dispatch;

use lazy_static::lazy_static;

use tokio::sync::mpsc;
use tracing::info;

use crate::core::CoreMsg;

pub mod empty;
pub mod kafka;

use empty::*;
use kafka::*;

lazy_static! {
	pub static ref SinkNames: Vec<&'static str> = vec!["kafka", "empty"];
}

#[enum_dispatch]
pub enum SinkerEnum {
	EmptySinker,
	KafkaSinker,
}

#[allow(async_fn_in_trait)]
#[enum_dispatch(SinkerEnum)]
pub trait Sinker {
	async fn sink(&self, r: mpsc::Receiver<CoreMsg>) -> anyhow::Result<()>;
}

pub fn get_sinker(name: &str, val: &serde_json::Value) -> anyhow::Result<SinkerEnum> {
	info!("sinker {}", name);

	match name.to_lowercase().as_str() {
		"kafka" => Ok(KafkaSinker::new(val)?.into()),
		"empty" => Ok(EmptySinker::new(val)?.into()),
		other => anyhow::bail!("unknown data sinker {}", other),
	}
}

#[cfg(test)]
mod my_test {
	use crate::biz::link::sink::{EmptySinker, SinkerEnum};

	#[tokio::test]
	pub async fn test_run() -> anyhow::Result<()> {
		let _sinker: SinkerEnum = EmptySinker::new(&serde_json::Value::Null)?.into();
		// sinker.sink(r)
		Ok(())
	}
}
