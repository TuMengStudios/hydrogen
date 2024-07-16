use tokio_metrics::TaskMonitor;

use lazy_static::lazy_static;
use tracing::instrument;

use crate::util::x_data;

use crate::core::AppData;
use crate::core::AppErr;

use crate::errcode;

use crate::extractor::RequestContext;

use crate::types::TokioMetrics;

pub struct MetricsHandler {}

lazy_static! {
	static ref task_monitor: TaskMonitor = TaskMonitor::new();
}

impl MetricsHandler {
	#[instrument(skip())]
	pub async fn metrics(req_ctx: RequestContext) -> Result<AppData<TokioMetrics>, AppErr> {
		let m = match task_monitor.clone().intervals().next() {
			Some(metrics) => metrics,
			None => {
				return Err(errcode::FETCH_METRICS_ERR.clone());
			}
		};
		let res = Ok(TokioMetrics::from_tokio_task_metrics(m));
		x_data(res)
	}
}

#[cfg(test)]
mod my_test {
	use std::time::Duration;

	use crate::handler::metrics::task_monitor;

	#[tokio::test]
	async fn my_test() {
		tokio::task::spawn(async { tokio::time::sleep(Duration::from_secs(2)).await });
		tokio::task::spawn(async { tokio::time::sleep(Duration::from_secs(2)).await });
		tokio::task::spawn(async { tokio::time::sleep(Duration::from_secs(2)).await });
		let m = task_monitor.clone().intervals().next().unwrap();
		println!("metrics {:#?}", m);
	}
}
