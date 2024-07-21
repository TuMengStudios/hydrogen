use tokio::sync::mpsc;

use tracing::info;
use tracing::instrument;

use super::Sinker;

use crate::core::CoreMsg;

pub struct EmptySinker {
	val: serde_json::Value,
}

impl EmptySinker {
	pub fn new(val: &serde_json::Value) -> anyhow::Result<EmptySinker> {
		Ok(Self { val: val.clone() })
	}
}

impl Sinker for EmptySinker {
	#[instrument(skip(self, r))]
	async fn sink(&self, mut r: mpsc::Receiver<CoreMsg>) -> anyhow::Result<()> {
		info!("start sink {}", serde_json::json!(self.val).to_string());
		while let Some(body) = r.recv().await {
			info!("receive data {:?}", body);
		}
		info!("close sender");
		Ok(())
	}
}
