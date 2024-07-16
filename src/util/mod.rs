use std::any::type_name;
use std::fmt::Debug;

use anyhow::Context;

use serde::de::DeserializeOwned;
use serde::Serialize;

use serde_json::json;

use tracing::debug;
use tracing::error;
use tracing::trace;

use crate::core::AppData;
use crate::core::AppErr;

pub fn from_val<T: DeserializeOwned>(val: &serde_json::Value) -> anyhow::Result<T> {
	trace!("decode value {:?} to type {}", json!(val).to_string(), type_name::<T>());

	let res = serde_json::from_value::<T>(val.clone()).with_context(|| {
		format!("decode val {:?} to type {}", json!(val).to_string(), type_name::<T>())
	})?;
	Ok(res)
}

pub fn x_data<T: Serialize + Debug>(data: Result<T, AppErr>) -> Result<AppData<T>, AppErr> {
	// debug!("process {:?}", data);
	match data {
		Ok(res) => {
			debug!("solve ok {:?}", res);
			Ok(AppData(res))
		}
		Err(err) => {
			error!("solve error {:?}", err);
			Err(err)
		}
	}
}
