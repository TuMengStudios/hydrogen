use anyhow::Context;

use serde::Deserialize;

use sqlx::MySqlPool;

use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio_context;

use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::warn;
use tracing::Instrument;
use tracing::Level;

use crate::biz::link::sink::get_sinker;
use crate::biz::link::sink::Sinker;
use crate::biz::link::source::get_source;
use crate::biz::link::source::Source;

use crate::biz::task_manger::add_task;
use crate::biz::task_manger::remove_task;
use crate::core::AppErr;
use crate::core::CoreMsg;

use crate::errcode;

use crate::model::task::TaskInfo;
use crate::model::task::TaskStatus;
use crate::model::task_log::TaskLog;

use crate::types::JsonParserOpt;
use crate::util::from_val;

use lepumk::ani::JsonParser;

use super::link::sink::SinkerEnum;
use super::link::source::SourceEnum;
use super::task_manger::contains_task;

const INTERVAL: u64 = 300;

#[derive(Debug, Deserialize)]
struct SinkArg {
	name: String,
	val: serde_json::Value,
}

impl SinkArg {
	fn new(val: &serde_json::Value) -> anyhow::Result<Self> {
		let s = from_val(val).with_context(|| {
			format!(
				"build {} value {:?}",
				std::any::type_name::<Self>(),
				serde_json::json!(val).to_string()
			)
		})?;
		Ok(s)
	}
}

impl SinkArg {
	fn get_name(&self) -> &str {
		&self.name
	}
	fn get_val(&self) -> &serde_json::Value {
		&self.val
	}
}

#[derive(Debug, Deserialize)]
struct SourceArg {
	name: String,
	val: serde_json::Value,
}

impl SourceArg {
	fn new(val: &serde_json::Value) -> anyhow::Result<Self> {
		let s = from_val(val).with_context(|| {
			format!(
				"build {} value {:?}",
				std::any::type_name::<Self>(),
				serde_json::json!(val).to_string(),
			)
		})?;
		Ok(s)
	}

	fn get_name(&self) -> &str {
		&self.name
	}

	fn get_val(&self) -> &serde_json::Value {
		&self.val
	}
}

pub struct Tasking {
	// todo
	sink: SinkerEnum,
	source: SourceEnum,
	task: TaskInfo,
	handle_num: AtomicI64,
	handle_err: AtomicI64,
}

impl Tasking {
	fn new(task: TaskInfo) -> anyhow::Result<Tasking> {
		info!("build task scheduler");
		let sink_arg = SinkArg::new(&task.dst_config)?;
		let sink = get_sinker(sink_arg.get_name(), sink_arg.get_val())?;
		let source_arg = SourceArg::new(&task.src_config)?;
		let source = get_source(source_arg.get_name(), source_arg.get_val())?;
		Ok(Self {
			sink,
			source,
			task,
			handle_num: AtomicI64::new(0),
			handle_err: AtomicI64::new(0),
		})
	}
}

impl Tasking {
	#[tracing::instrument(skip(task, conn))]
	pub async fn start_task(task: TaskInfo, conn: MySqlPool) -> Result<(), AppErr> {
		if contains_task(task.id) {
			info!("task is running {}", task.id);
			return Err(errcode::TASKING_ALREADY_RUNNING.clone());
		}

		let parent = tracing::Span::none();
		let span = tracing::span!(parent:&parent,Level::ERROR,module_path!(),"task_id"=task.id);
		let mut tasking = match Tasking::new(task.clone()) {
			Ok(tasking) => tasking,
			Err(err) => {
				error!("build task error {:?}", err);
				let mut task = task.clone();
				task.status = TaskStatus::Stop.get_status();
				let _ = TaskInfo::update_task(&conn, &mut task).await;
				let mut task_log = TaskLog::new(task.id, &format!("error {:?}", err));
				let ins_res = TaskLog::insert_task_log(&conn, &mut task_log).await;
				info!("insert log result {:?}", ins_res);
				return Err(errcode::TASKING_START_ERR.clone());
			}
		};

		tokio::task::spawn(
			async move {
				let _r = tasking.start_job_internal(conn).await;
			}
			.instrument(span),
		);

		Ok(())
	}
}

impl Tasking {
	//
	async fn start_job_internal(&mut self, conn: MySqlPool) -> anyhow::Result<()> {
		// build channel
		let (s1, r1) = tokio::sync::mpsc::channel(6);
		let (s2, r2) = tokio::sync::mpsc::channel(6);
		self.task.status = TaskStatus::Running.get_status();

		let _ = TaskInfo::update_task(&conn, &mut self.task).await;
		let parser = from_val::<JsonParserOpt>(&self.task.parser_config)
			.with_context(|| format!("build json parser opt error {:?}", self.task.parser_config))?
			.to_parser();

		let (_, mut handle) = tokio_context::context::Context::new();
		let mut ctx = handle.spawn_ctx();
		add_task(self.task.id, handle);
		let res = tokio::select! {
			_ = ctx.done() => {

				Err(anyhow::anyhow!("use cancel task"))
			},
			res = self.sink.sink(r2) => {
				//
				res
			},
			res = self.source.source(s1) => {
				res
			},
			res= self.handle_msg(parser,r1,s2) => {
				res
			},
			res = self.update_task_heartbeat(conn.clone(), self.task.id) => {
				res
			}
		};

		// task has cancel or error
		remove_task(self.task.id);
		warn!("stop__task {res:?} {}", self.task.id);

		let add_handle_num = self
			.handle_num
			.fetch_add(-self.handle_num.fetch_add(0, Ordering::Relaxed), Ordering::Relaxed);

		//
		let add_handle_err = self
			.handle_err
			.fetch_add(-self.handle_err.fetch_add(0, Ordering::Relaxed), Ordering::Relaxed);

		// update task status
		let _ = TaskInfo::update_meta(&conn, self.task.id, add_handle_num, add_handle_err).await;

		match res {
			Ok(res) => {
				info!("task success {:?}", res);
				let mut task_log = TaskLog::new(self.task.id, "finished success");
				let _ = TaskLog::insert_task_log(&conn, &mut task_log).await;
				// update status as stop
				self.task.status = TaskStatus::Stop.get_status();
				// updated updated_at time
				self.task.updated_at = chrono::Local::now().timestamp();

				let _ = TaskInfo::update_task(&conn, &mut self.task).await;
			}
			Err(err) => {
				// error
				error!("task error {:?}", err);
				let mut task = self.task.clone();
				task.status = TaskStatus::Stop.get_status();
				let _ = TaskInfo::update_task(&conn, &mut task).await;
				let mut task_log = TaskLog::new(task.id, &format!("error {:?}", err));
				let ins_res = TaskLog::insert_task_log(&conn, &mut task_log).await;
				info!("insert log result {:?}", ins_res);
			}
		};
		Ok(())
	}
}
// type clean_job = fn(i64, &MySqlPool);

impl Tasking {
	// update task heartbeat
	// use local timestamp as heartbeat
	#[tracing::instrument(skip(conn, self))]
	async fn update_task_heartbeat(&self, conn: MySqlPool, id: i64) -> anyhow::Result<()> {
		//
		info!("update task heartbeat task sub job {id}");
		let mut ticker = tokio::time::interval(Duration::from_secs(INTERVAL));
		let mut err_cnt: i32 = 0;

		// ...
		loop {
			let _i = ticker.tick().await;
			// ..
			let add_handle_num = self
				.handle_num
				.fetch_add(-self.handle_num.fetch_add(0, Ordering::Relaxed), Ordering::Relaxed);

			//
			let add_handle_err = self
				.handle_err
				.fetch_add(-self.handle_err.fetch_add(0, Ordering::Relaxed), Ordering::Relaxed);

			// update task status
			let res = TaskInfo::update_meta(&conn, id, add_handle_num, add_handle_err).await;
			match res {
				Ok(_) => {
					// todo
					debug!("update heartbeat success {}", id);
					err_cnt = 0;
				}
				Err(err) => {
					// ..
					error!("update heartbeat task id {} error {:?}", id, err);
					err_cnt += 1;
					if err_cnt >= 10 {
						break;
					}
				}
			}
		}
		Ok(())
	}
}

impl Tasking {
	async fn handle_msg(
		&self,
		p: JsonParser,
		mut receiver: mpsc::Receiver<CoreMsg>,
		sender: mpsc::Sender<CoreMsg>,
	) -> anyhow::Result<()> {
		while let Some(mut msg) = receiver.recv().await {
			self.handle_num.fetch_add(1, Ordering::Relaxed);
			match p.run(msg.get_raw_msg()).await {
				Ok(result) => {
					msg.result = result;
				}
				Err(err) => {
					error!("handle msg {} error {:?}", msg.get_raw_msg(), err);
					self.handle_err.fetch_add(1, Ordering::Relaxed);
					continue;
				}
			};
			msg = msg.with_raw_keys(p.0.get_keys().clone());
			sender.send(msg).await?;
		}

		info!("close sender");
		Ok(())
	}
}
