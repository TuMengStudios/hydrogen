use std::collections::HashMap;

use anyhow::Context;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::HeaderMap;
use axum::http::Method;
use axum::http::Uri;
use axum::http::Version;

use serde::de::DeserializeOwned;
use serde::Serialize;

use serde_json::json;

// #[derive(Debug)]
pub struct RequestContext {
	pub method: Method,
	pub uri: Uri,
	pub header: HeaderMap,
	pub version: Version,
	pub data: HashMap<String, serde_json::Value>,
}

// user define debug for RequestContext
impl std::fmt::Debug for RequestContext {
	//
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("RequestContext")
			.field("method", &self.method)
			.field("uri", &self.uri)
			// .field("header", &self.header)
			.field("version", &self.version)
			// .field("data", &self.data)
			.finish()
	}
}

impl RequestContext {
	pub fn get_data<T: DeserializeOwned>(&self, key: &str) -> anyhow::Result<T> {
		match self.data.get(key) {
			Some(val) => {
				// parser value from data expect type name
				let res = serde_json::from_value::<T>(val.clone()).with_context(|| {
					let type_name = std::any::type_name::<T>();
					format!(
						"parser value {:?} expect type {} but get {}",
						json!(val).to_string(),
						self.check_value_type(val),
						type_name,
					)
				})?;
				// response val
				Ok(res)
			}
			None => anyhow::bail!("not found {key}"),
		}
	}

	fn check_value_type(&self, val: &serde_json::Value) -> &str {
		match val {
			serde_json::Value::Null => "null",
			serde_json::Value::Bool(_) => "boolean",
			serde_json::Value::Number(_) => "number",
			serde_json::Value::String(_) => "string",
			serde_json::Value::Array(_) => "array",
			serde_json::Value::Object(_) => "object",
		}
	}

	pub fn set_data<T: Serialize>(&mut self, key: &str, data: T) {
		let val = serde_json::to_value(data).unwrap();
		self.data.insert(key.to_owned(), val);
	}
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for RequestContext
where
	S: Send + Sync,
{
	type Rejection = &'static str;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
		Ok(RequestContext {
			data: HashMap::new(),
			uri: parts.uri.clone(),
			version: parts.version,
			method: parts.method.clone(),
			header: parts.headers.clone(),
		})
	}
}
