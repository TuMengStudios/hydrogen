use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use tokio_context::context::Handle;

use tracing::error;
use tracing::info;

use lazy_static::lazy_static;

lazy_static! {
	static ref TASK_MANAGER: TaskManager = TaskManager::new();
}

pub fn contains_task(id: i64) -> bool {
	TASK_MANAGER.contains_task(id)
}

pub fn remove_task(id: i64) {
	TASK_MANAGER.remove_task(id)
}

pub fn running_task() -> Vec<i64> {
	TASK_MANAGER.task_list()
}

pub fn add_task(id: i64, handle: Handle) {
	TASK_MANAGER.add_task(id, TaskContext::new(handle))
}

struct TaskManager {
	tasks: Arc<Mutex<HashMap<i64, TaskContext>>>,
}

impl TaskManager {
	#[allow(clippy::new_without_default)]
	fn new() -> Self {
		let data = HashMap::new();
		Self { tasks: Arc::new(Mutex::new(data)) }
	}

	// remove task
	fn remove_task(&self, id: i64) {
		let a = self.tasks.clone();
		let mut data = a.lock().unwrap();
		if data.contains_key(&id) {
			info!("remove task id {:?}", id);
			let _ = data.remove(&id);
		} else {
			error!("not found task {id}");
		}
	}

	// add task
	#[tracing::instrument(skip(self, id, task))]
	fn add_task(&self, id: i64, task: TaskContext) {
		let a = self.tasks.clone();
		let mut data = a.lock().unwrap();
		if data.contains_key(&id) {
			info!("twice id {}", id);
			return;
		}
		info!("add task {:?}", id);
		data.insert(id, task);
	}

	// get running task list
	fn task_list(&self) -> Vec<i64> {
		let a = self.tasks.clone();
		let data = a.lock().unwrap();
		let res = data.keys().copied().collect();
		res
	}

	pub fn contains_task(&self, id: i64) -> bool {
		let data = self.tasks.clone();
		let c = data.lock().unwrap();
		c.contains_key(&id)
	}
}

#[allow(unused)]
struct TaskContext {
	// manager task tree handle
	handle: tokio_context::context::Handle,
}

impl TaskContext {
	// new task context
	fn new(handle: Handle) -> Self {
		Self {
			// init TaskContext
			handle,
		}
	}
}
