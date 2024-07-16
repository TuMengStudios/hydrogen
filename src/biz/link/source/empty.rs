use tokio::sync::mpsc;

use tracing::info;
use tracing::instrument;

use crate::core::CoreMsg;

use super::Source;

pub struct EmptySource {
	val: serde_json::Value,
}

impl EmptySource {
	pub fn new(val: &serde_json::Value) -> anyhow::Result<EmptySource> {
		Ok(Self { val: val.clone() })
	}
}

impl Source for EmptySource {
	#[instrument(skip(self, s))]
	async fn source(&self, s: mpsc::Sender<CoreMsg>) -> anyhow::Result<()> {
		info!("start source  {}", serde_json::json!(self.val).to_string());
		let _ = s.send(CoreMsg::default().with_result(vec![])).await;
		Ok(())
	}
}
