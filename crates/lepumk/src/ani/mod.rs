use std::collections::HashMap;
use std::collections::HashSet;
use std::vec;

use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Map;

use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::instrument;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ParserOptions {
	// filed join sep
	// pub(crate) sep: String,
	sep: String,
	// max depth, over is exit
	max_depth: i32,
	// fold key and then not extend the field
	fold: HashSet<String>,
	// ignore key
	ignore: HashSet<String>,
	// default value
	default_value: HashMap<String, serde_json::Value>,
	// use strict mode just parser contain in key
	strict: bool,
	// keys from debug text
	keys: HashSet<String>,
}

impl ParserOptions {
	// build default parserOptions
	pub fn fmt() -> Self {
		Self::default()
	}

	// build JsonParser
	pub fn init(self) -> JsonParser {
		JsonParser(self)
	}

	// with sep
	pub fn with_sep(self, sep: &str) -> Self {
		Self { sep: sep.to_owned(), ..self }
	}

	// fold the value and then not parser this  value next
	pub fn with_fold(self, fold: HashSet<String>) -> Self {
		Self { fold, ..self }
	}

	// set ignore keys
	pub fn with_ignore(self, ignore: HashSet<String>) -> Self {
		Self { ignore, ..self }
	}

	// set max depth
	// default 0 is ignored
	pub fn with_max_depth(self, max_depth: i32) -> Self {
		Self { max_depth, ..self }
	}

	// set run strict mode;
	// if strict mode just filter keys
	pub fn with_strict_mode(self, mode: bool) -> Self {
		Self { strict: mode, ..self }
	}

	// set default value if value is null
	pub fn with_default_value(
		self,
		default_value: HashMap<String, serde_json::Value>,
	) -> Self {
		Self { default_value, ..self }
	}

	// set keys
	pub fn with_keys(self, keys: HashSet<String>) -> Self {
		Self { keys, ..self }
	}

	pub fn get_keys(&self) -> &HashSet<String> {
		&self.keys
	}
}

impl ParserOptions {
	// check key is in ignore or else
	pub fn contains_ignore(&self, key: &str) -> bool {
		self.ignore.contains(key)
	}

	// check key is fold or not, if true not parser and then
	pub fn contains_fold(&self, key: &str) -> bool {
		self.fold.contains(key)
	}

	// if value is null get default
	pub fn get_default(&self, key: &str) -> Option<&serde_json::Value> {
		self.default_value.get(key)
	}

	// check key is in keys container  if in strict mode not contain will drop this key and value
	pub fn contain_key(&self, key: &str) -> bool {
		self.keys.contains(key)
	}

	// check run in strict mode or not
	pub fn strict_mode(&self) -> bool {
		self.strict
	}
	// get sep
	pub fn get_sep(&self) -> &str {
		&self.sep
	}
}

#[derive(Debug)]
#[allow(unused)]
pub struct JsonParser(pub ParserOptions);

impl JsonParser {
	fn join_key(&self, pre_key: &str, curr_key: &str) -> String {
		debug!(
			"pre key {pre_key}, sep {}, curr_key {curr_key}",
			self.0.get_sep()
		);

		if pre_key.is_empty() {
			curr_key.to_owned()
		} else {
			format!("{pre_key}{}{curr_key}", self.0.get_sep())
		}
	}
}

impl JsonParser {
	#[tracing::instrument(skip(self, s))]
	pub async fn run(
		&self,
		s: &str,
	) -> anyhow::Result<Vec<HashMap<String, serde_json::Value>>> {
		debug!("parser value {}", s);

		let val = serde_json::from_str::<serde_json::Value>(s)?;
		let key = self.join_key("", "");
		// check in strict or not
		if self.0.strict_mode() && !self.0.contain_key(&key) {
			debug!(
				"strict mode or not  {} key is {} {}",
				self.0.strict_mode(),
				key,
				self.0.contain_key(&key)
			);

			return Ok(vec![]);
		}
		// check key is ignore
		if self.0.contains_ignore(&key) {
			debug!("expected key {} in keys", key);
			//
			return Ok(vec![]);
		}

		// check is key is fold
		if self.0.contains_fold(&key) {
			let mut m = HashMap::new();
			m.insert(key, val);
			return Ok(vec![m]);
		}

		// strict mode but not contain key
		if self.0.strict_mode() && !self.0.contain_key(&key) {
			debug!("run in strict mode but not found key {} in keys", key);
			return Ok(vec![]);
		}

		match val {
			serde_json::Value::Array(arr) => {
				let m: HashMap<String, serde_json::Value> = HashMap::new();

				self.parser_array(&arr, &key, &m, 0)
			}

			serde_json::Value::Object(obj) => {
				let m: HashMap<String, serde_json::Value> = HashMap::new();
				self.parser_object(&obj, &key, &m, 0)
			}

			pri => {
				let mut m = HashMap::new();
				m.insert(key, pri);
				Ok(vec![m])
			}
		}
	}

	#[allow(clippy::ptr_arg)]
	#[instrument(skip(self, arr, pre_key, depth, curr))]
	fn parser_array(
		&self,
		arr: &Vec<serde_json::Value>,
		pre_key: &str,
		curr: &HashMap<String, serde_json::Value>,
		depth: i32,
	) -> anyhow::Result<Vec<HashMap<String, serde_json::Value>>> {
		//
		if depth > self.0.max_depth && self.0.max_depth > 0 {
			anyhow::bail!(
				"depth({}) is larger than max_depth({})",
				depth,
				self.0.max_depth
			)
		}

		let mut res = Vec::new();

		let full_key = self.join_key(pre_key, "");
		if arr.is_empty() {
			debug!("parser array but empty");
			res.push(curr.clone());
			let val = if let Some(val) = self.0.get_default(&full_key) {
				debug!("empty array use default value {:?}", val);
				val.clone()
			} else {
				debug!("empty array use []");
				serde_json::Value::Array(arr.clone())
			};

			for item in res.iter_mut() {
				item.insert(full_key.to_owned(), val.clone());
			}
			return Ok(res);
		}

		if self.0.strict_mode() && !self.0.contain_key(&full_key) {
			debug!("run in strict mode not found key {}", full_key);
			return Ok(vec![curr.clone()]);
		}

		if self.0.contains_ignore(&full_key) {
			debug!("ignore key {}", full_key);
			return Ok(vec![curr.clone()]);
		}

		if self.0.contains_fold(&full_key) {
			debug!("fold key {}", full_key);
			let mut data = curr.clone();
			data.insert(
				full_key.to_owned(),
				serde_json::Value::Array(arr.clone()),
			);
			return Ok(vec![data]);
		}

		for val in arr.iter() {
			if self.0.contains_fold(&full_key) {
				let mut data = curr.clone();
				data.insert(full_key.to_owned(), val.clone());
				res.push(data);
				continue;
			}

			match val {
				serde_json::Value::Array(arr) => {
					debug!("parser occur arr {:?}", arr);
					match self.parser_array(arr, &full_key, curr, depth + 1) {
						Ok(mut r) => {
							// todo
							res.append(&mut r);
						}
						Err(err) => {
							//
							error!("parser array error {:?}", err);
						}
					}
				}
				serde_json::Value::Object(obj) => {
					debug!("parser occur obj {:?}", obj);
					match self.parser_object(obj, &full_key, curr, depth + 1) {
						Ok(mut r) => {
							res.append(&mut r);
						}
						Err(err) => {
							error!("parser array error {:?}", err);
						}
					}
				}
				//
				pri => {
					debug!("value {:?}", pri);
					if pri.is_null() {
						if let Some(default_value) =
							self.0.get_default(&full_key)
						{
							let mut data = curr.clone();
							data.insert(
								full_key.clone(),
								default_value.clone(),
							);
							res.push(data);
							continue;
						}
					}

					let mut data = curr.clone();
					data.insert(full_key.to_owned(), pri.clone());
					res.push(data);
				}
			}
		}

		Ok(res)
	}

	#[instrument(skip(self, obj, curr, depth, pre_key))]
	fn parser_object(
		&self,
		obj: &Map<String, serde_json::Value>,
		pre_key: &str,
		curr: &HashMap<String, serde_json::Value>,
		depth: i32,
	) -> anyhow::Result<Vec<HashMap<String, serde_json::Value>>> {
		//...
		debug!("obj {}", json!(obj).to_string());
		//...
		if depth > self.0.max_depth && self.0.max_depth > 0 {
			error!(
				"depth({}) is larger than max_depth({})",
				depth, self.0.max_depth
			);
			anyhow::bail!(
				"depth({}) is larger than max_depth({})",
				depth,
				self.0.max_depth
			)
		}
		let mut res: Vec<HashMap<String, serde_json::Value>> = vec![];
		//  iter map  and then check value type
		for (key, val) in obj.iter() {
			let full_key = self.join_key(pre_key, key);

			// if is strict mode and keys not contains key
			if self.0.strict_mode() && !self.0.contain_key(&full_key) {
				debug!("run in strict mode occur key {}", full_key);
				continue;
			}

			// if this key is ignore will drop this key and value
			if self.0.contains_ignore(&full_key) {
				debug!("ignore full key {}", full_key);
				continue;
			}

			// if fold this value will not expand
			if self.0.contains_fold(&full_key) {
				debug!("full_key {} fold", full_key);
				if res.is_empty() {
					res.push(curr.clone());
				}
				//  check val is null
				let new_val = if val.is_null() {
					match self.0.get_default(&full_key) {
						Some(default_value) => {
							debug!(
								"default value  full key {} default value {:?}",
								full_key, default_value
							);
							// default value
							default_value
						}
						None => {
							// return null
							debug!("occur null value but not set in default_values {}", full_key);
							val
						}
					}
				} else {
					val
				};

				// iter result vec and push key value into item
				for item in res.iter_mut() {
					item.insert(full_key.to_owned(), new_val.clone());
				}

				continue;
			}

			match val {
				serde_json::Value::Array(arr) => {
					if res.is_empty() {
						res.push(curr.clone());
					}
					let mut temp_res: Vec<HashMap<String, serde_json::Value>> =
						vec![];

					for item in res.iter() {
						let rr =
							self.parser_array(arr, &full_key, item, depth + 1);

						match rr {
							Ok(mut r) => {
								temp_res.append(&mut r);
							}
							Err(err) => {
								error!("parser array error {:?}", err);
							}
						}
					}

					res = temp_res;
				}
				serde_json::Value::Object(obj) => {
					// check is empty
					if res.is_empty() {
						res.push(curr.clone());
					}

					let mut temp_res = vec![];

					for item in res.iter() {
						let rr =
							self.parser_object(obj, &full_key, item, depth + 1);
						match rr {
							Ok(mut r) => {
								// extend array list
								temp_res.append(&mut r);
							}

							Err(err) => {
								// occur error
								error!(
									"parser error {} {:?}",
									json!(obj).to_string(),
									err
								);
							}
						};
					}
					res = temp_res;
				}

				pri => {
					// base value type like bool, number, string and null
					if res.is_empty() {
						res.push(curr.clone());
					}
					//  check val is null
					let new_val = if pri.is_null() {
						match self.0.get_default(&full_key) {
							Some(default_value) => {
								debug!(
									"default value  full key {} default value {:?}",
									full_key, default_value
								);
								// default value
								default_value
							}
							None => {
								// return null
								pri
							}
						}
					} else {
						pri
					};

					for item in res.iter_mut() {
						item.insert(full_key.to_owned(), new_val.clone());
					}
					continue;
				}
			}
		}

		// check res is empty if return [curr.clone()]
		if res.is_empty() {
			res.push(curr.clone());
		}

		if obj.is_empty() {
			let full_key = pre_key.to_owned();
			for item in res.iter_mut() {
				let val = if let Some(val) = self.0.get_default(&full_key) {
					val.clone()
				} else {
					serde_json::Value::Object(obj.clone())
				};

				item.insert(full_key.to_owned(), val);
			}
		}

		Ok(res)
	}
}

// parser input property
// map/array/string,bool,number,null
impl JsonParser {
	#[instrument(skip(self, s))]
	pub async fn property(&self, s: &str) -> anyhow::Result<Property> {
		debug!("input value {}", s);
		let val = serde_json::from_str::<serde_json::Value>(s)?;

		let value_type = self.value_type(&val);
		let key = self.join_key("", "");

		let item = match &val {
			serde_json::Value::Array(arr) => {
				debug!(
					"value type {} arr {}",
					value_type,
					json!(arr).to_string()
				);
				// dive arr sub item
				PropertyItem::new(
					key,
					value_type.to_owned(),
					self.property_arr(arr),
				)
			}

			serde_json::Value::Object(obj) => {
				debug!(
					"value type {} obj {}",
					value_type,
					json!(obj).to_string()
				);
				// dive object item
				PropertyItem::new(
					key.clone(),
					value_type.to_owned(),
					self.property_object(obj),
				)
			}
			_pri => {
				debug!(
					"pri type {} val {}",
					value_type,
					json!(val.clone()).to_string()
				);
				PropertyItem::new(key, value_type.to_owned(), vec![])
			}
		};

		Ok(Property::new(self.0.sep.clone(), val.clone(), item))
	}

	#[instrument(skip(self, obj))]
	fn property_object(
		&self,
		obj: &Map<String, serde_json::Value>,
	) -> Vec<PropertyItem> {
		debug!("property_object {}", json!(obj).to_string());

		let mut res = vec![];

		for (key, val) in obj {
			debug!("key {}, value {}", key, json!(val).to_string());
			let value_type = self.value_type(val);

			match val {
				// parser object
				serde_json::Value::Object(obj) => {
					res.push(PropertyItem::new(
						key.clone(),
						value_type.to_owned(),
						self.property_object(obj),
					));
				}
				serde_json::Value::Array(arr) => {
					res.push(PropertyItem::new(
						key.clone(),
						value_type.to_owned(),
						self.property_arr(arr),
					));
				}
				_ => {
					res.push(PropertyItem::new(
						key.to_owned(),
						value_type.to_owned(),
						vec![],
					));
				}
			}
		}

		return res;
	}
	#[instrument(skip(self))]
	fn property_arr(&self, arr: &Vec<serde_json::Value>) -> Vec<PropertyItem> {
		debug!("property_arr {} len({})", json!(arr).to_string(), arr.len());

		let mut res = vec![];

		match arr.first() {
			Some(val) => {
				let value_type = self.value_type(val);
				let key = "";
				debug!(
					"property_arr val {} node type {}",
					json!(val).to_string(),
					value_type
				);

				match val {
					serde_json::Value::Object(obj) => {
						res.push(PropertyItem::new(
							key.to_owned(),
							value_type.to_owned(),
							self.property_object(obj),
						));
					}

					serde_json::Value::Array(arr) => {
						res.push(PropertyItem::new(
							key.to_owned(),
							value_type.to_owned(),
							self.property_arr(arr),
						));
					}
					_ => {
						res.push(PropertyItem::new(
							key.to_owned(),
							value_type.to_owned(),
							vec![],
						));
					}
				}
			}
			None => {
				info!("key empty array")
			}
		}

		res
	}

	fn value_type(&self, val: &serde_json::Value) -> &'static str {
		match &val {
			serde_json::Value::Null => DataType::NULL,
			serde_json::Value::Bool(_) => DataType::BOOLEAN,
			serde_json::Value::Number(_) => DataType::NUMBER,
			serde_json::Value::String(_) => DataType::STRING,
			serde_json::Value::Array(_) => DataType::ARRAY,
			serde_json::Value::Object(_) => DataType::OBJECT,
		}
	}
}

#[allow(non_snake_case)]
mod DataType {
	// null value
	pub(crate) const NULL: &str = "null";
	// boolean
	pub(crate) const BOOLEAN: &str = "boolean";
	// number like int, float
	pub(crate) const NUMBER: &str = "number";
	// string
	pub(crate) const STRING: &str = "string";
	// array
	pub(crate) const ARRAY: &str = "array";
	// object
	pub(crate) const OBJECT: &str = "object";
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PropertyItem {
	// key
	node_name: String,
	// value type
	value_type: String,

	// sub node
	props: Vec<PropertyItem>,

	op: String,
}

#[warn(dead_code)]
#[allow(unused)]
pub enum PropertyItemOp {
	Keep,   // hold this property
	Fold,   // fold this property if is array or object
	Ignore, // ignore this property
}

impl PropertyItemOp {
	pub fn get_value(&self) -> String {
		match *self {
			PropertyItemOp::Fold => "fold".to_owned(),
			PropertyItemOp::Keep => "keep".to_owned(),
			PropertyItemOp::Ignore => "ignore".to_owned(),
		}
	}
}

impl PropertyItem {
	pub fn new(
		node_name: String,
		value_type: String,
		props: Vec<PropertyItem>,
	) -> Self {
		Self {
			node_name,
			value_type,
			props,
			op: PropertyItemOp::Keep.get_value(),
		}
	}

	#[allow(unused)]
	fn with_op(self, op: PropertyItemOp) -> Self {
		Self { op: op.get_value(), ..self }
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Property {
	// sep value like "_", "__" or other user define sep
	sep: String,
	// parser value
	value: serde_json::Value,
	// property node tree
	item: PropertyItem,
}

//
impl Property {
	pub fn new(
		sep: String,
		value: serde_json::Value,
		item: PropertyItem,
	) -> Self {
		Self { sep, value, item }
	}
}

#[cfg(test)]
mod tests {
	use serde_json::json;
	use std::collections::HashMap;
	use std::collections::HashSet;
	use tracing::debug;
	use tracing::error;
	use tracing::info;
	use tracing::Instrument;
	use tracing::Level;
	use tracing_subscriber::fmt::format::FmtSpan;
	use tracing_subscriber::fmt::time::LocalTime;

	use crate::ani::ParserOptions;
	use crate::ani::PropertyItem;

	lazy_static::lazy_static! {
		static ref _Str : &'static str = {
			tracing_subscriber::fmt()
			.with_ansi(true)
			.with_timer(LocalTime::rfc_3339())
			.with_span_events(FmtSpan::CLOSE)
			.with_line_number(true)
			.with_file(true)
			.with_max_level(tracing::Level::DEBUG)
			.init();

			"xx"
		};
	}

	#[tokio::test]
	async fn test_debug() {
		let _ = _Str.clone();
		let v = PropertyItem::default();
		info!("test_debug_val {:?}", v);
	}

	#[tokio::test]
	async fn test_with_max_depth() {
		let _d = _Str.clone();
		let opt = ParserOptions::fmt();
		assert_eq!(opt.max_depth, 0);
		let opt = ParserOptions::fmt().with_max_depth(10);
		assert_eq!(opt.max_depth, 10)
	}

	#[tokio::test]
	async fn test_with_strict_mode() {
		let _d = _Str.clone();
		let opt = ParserOptions::fmt();
		assert_eq!(opt.strict, false);
		let opt = ParserOptions::fmt().with_strict_mode(true);
		assert_eq!(opt.strict, true);
		let opt = ParserOptions::fmt().with_strict_mode(false);
		assert_eq!(opt.strict, false);
	}

	#[tokio::test]
	async fn test_with_fold() {
		let _d = _Str.clone();
		let fold = HashSet::new();
		let opt = ParserOptions::fmt().with_fold(fold.clone());
		assert_eq!(opt.fold, fold.clone());
	}

	#[tokio::test]
	async fn test_with_ignore() {
		let _d = _Str.clone();
		let mut ignore = HashSet::new();
		let opt = ParserOptions::fmt().with_ignore(ignore.clone());
		assert_eq!(opt.ignore, ignore.clone());
		ignore.insert("abc".to_owned());

		let opt = ParserOptions::fmt().with_ignore(ignore.clone());
		assert_eq!(opt.ignore, ignore.clone());
		assert_ne!(opt.ignore, HashSet::new());
	}

	#[tokio::test]
	async fn test_with_keys() {
		let _d = _Str.clone();

		let mut keys = HashSet::new();
		let opt = ParserOptions::fmt().with_keys(keys.clone());
		assert_eq!(opt.keys, keys.clone());
		keys.insert("foo/baz".to_owned());

		let opt = ParserOptions::fmt().with_keys(keys.clone());
		assert_eq!(opt.keys, keys.clone());
		assert_ne!(opt.keys, HashSet::new());
	}

	#[tokio::test]
	async fn test_join_key() {
		let _ = _Str.clone();
		let p = ParserOptions::fmt().with_sep("_").init();
		let key = p.join_key("hello", "world");
		assert_eq!(key, "hello_world".to_owned());

		let key = p.join_key("", "key");
		assert_eq!(key, "key".to_owned());

		let p = ParserOptions::fmt().with_sep("").init();
		let key = p.join_key("hello", "World");
		assert_eq!(key, "helloWorld".to_owned());
	}

	// #[test]
	#[tokio::test]
	async fn test_with_sep() {
		let _ = _Str.clone();
		let opt = ParserOptions::fmt();

		assert_eq!(&opt.sep, "");

		let s = "_".to_owned();
		let opt = ParserOptions::fmt().with_sep(s.as_str());
		assert_eq!(&opt.sep, "_");
	}

	#[tokio::test]
	#[tracing::instrument]
	async fn test_fold() -> anyhow::Result<()> {
		let _ = _Str.clone();

		let mut fold = HashSet::new();
		fold.insert("".to_owned());
		let mut keys = HashSet::new();
		keys.insert("".to_owned());

		let p = ParserOptions::fmt()
			.with_sep("_")
			.with_fold(fold)
			.with_keys(keys)
			.with_ignore(HashSet::new())
			.with_strict_mode(true)
			.init();

		let s = r#"{
            "key":"value"
            }"#;

		let res = p.run(s).await?;
		info!("test_fold=== {:?}", res);
		Ok(())
	}

	#[tokio::test]
	async fn test_opt_serialize() {
		let _ = _Str.clone();

		let mut fold = HashSet::new();
		fold.insert("hello".to_owned());
		fold.insert("key".to_owned());
		fold.insert("value".to_owned());
		let opt = ParserOptions::fmt().with_fold(fold);
		println!("{}", json!(opt).to_string());
	}

	#[tokio::test]
	async fn test_opt_des() -> anyhow::Result<()> {
		let _ = _Str.clone();

		let s = r#"{
        "default_value":{}
        ,"fold":["hello","value","key"]
        ,"ignore":[]
        ,"keys":[]
        ,"max_depth":0
        ,"sep":""
        ,"strict":false
        }"#;

		let opt = serde_json::from_str::<ParserOptions>(s)?;
		assert_eq!(opt.fold.contains("key"), true);
		assert_eq!(opt.fold.contains("fuzz"), false);
		assert_eq!(opt.sep, "".to_owned());
		assert_eq!(opt.ignore.len(), 0);
		assert_eq!(opt.keys.len(), 0);
		assert_eq!(opt.max_depth, 0);
		println!("opt {:?}", opt);
		Ok(())
	}

	#[tokio::test]
	async fn test_parser_simple_object() -> anyhow::Result<()> {
		let _ = _Str.clone();

		let p = ParserOptions::fmt().with_max_depth(10).with_sep("_").init();

		let s = r#"{
        "key":"value"
        ,"number": 1
        }"#;

		let res = p.run(s).await?;

		assert_eq!(res.len(), 1);
		assert_eq!(
			res[0].get("key"),
			Some(&serde_json::Value::String("value".to_owned()))
		);

		assert_eq!(res[0].get("key").unwrap().is_string(), true);
		assert_eq!(res[0].len(), 2);
		Ok(())
	}

	#[tokio::test]
	async fn test_parser_complex_object() -> anyhow::Result<()> {
		let _ = _Str.clone();

		let mut ignore = HashSet::new();
		ignore.insert("key".to_owned());

		let p = ParserOptions::fmt()
			.with_fold(HashSet::new())
			.with_keys(HashSet::new())
			.with_strict_mode(false)
			.with_default_value(HashMap::new())
			.with_max_depth(10)
			.with_ignore(ignore)
			.with_sep("_")
			.init();

		let s = r#"{
        "key":"value"
        ,"number": 1
        }"#;

		let res = p.run(s).await?;

		assert_eq!(res.len(), 1);
		assert_eq!(res[0].len(), 1);
		Ok(())
	}

	#[tokio::test]
	async fn test_parser_complex_default_value() -> anyhow::Result<()> {
		let _ = _Str.clone();

		let mut ignore = HashSet::new();
		ignore.insert("key".to_owned());

		let mut default_value = HashMap::new();
		default_value
			.insert("default".to_owned(), serde_json::Value::Bool(true));

		let p = ParserOptions::fmt()
			.with_max_depth(10)
			.with_ignore(ignore)
			.with_default_value(default_value)
			.with_keys(HashSet::new())
			.with_fold(HashSet::new())
			.with_strict_mode(false)
			.init();

		let s = r#"{
        "key":"value"
        ,"number": 1
        , "default":null
        }"#;

		let res = p.run(s).await?;

		assert_eq!(res.len(), 1);
		assert_eq!(res[0].len(), 2);
		assert_eq!(res[0].get("default"), Some(&serde_json::Value::Bool(true)));
		Ok(())
	}

	#[tokio::test]
	async fn test_parser_complex_fold() -> anyhow::Result<()> {
		let _ = _Str.clone();

		let mut ignore = HashSet::new();
		ignore.insert("key".to_owned());

		let mut fold = HashSet::new();
		fold.insert("fold".to_owned());

		let p = ParserOptions::fmt()
			.with_keys(HashSet::new())
			.with_default_value(HashMap::new())
			.with_max_depth(10)
			.with_fold(fold)
			.with_ignore(ignore)
			.init();

		let s = r#"{
        "key":"value"
        ,"number": 1
        , "fold": {"hello":"world"}
        }"#;

		let res = p.run(s).await?;

		assert_eq!(res.len(), 1);
		assert_eq!(res[0].len(), 2);
		let mut m = serde_json::Map::new();
		m.insert(
			"hello".to_owned(),
			serde_json::Value::String("world".to_owned()),
		);
		assert_eq!(res[0].get("fold"), Some(&serde_json::Value::Object(m)));

		Ok(())
	}

	#[tokio::test]
	async fn test_parser_complex_array() -> anyhow::Result<()> {
		let _ = _Str.clone();

		use nid::Nanoid;
		let id: Nanoid = Nanoid::new();

		let sp = tracing::span!(
			Level::ERROR,
			"test_parser_complex_array",
			"task_id" = format!("{}", id),
		);
		// let _guard = sp.enter();
		let p = ParserOptions::fmt().with_sep("_").with_max_depth(100).init();

		let s = r#"{
        "key":"value"
        ,"complex_array": [
            {
                "number":1
                , "nest":[1,2]
                , "nest_obj": {"key":"value", "nest_arr":[1,2,3]}
            }
         ]
      }"#;

		let res = p.run(s).instrument(sp).await?;
		info!("res==  {}", json!(res).to_string());
		assert_eq!(res.len(), 6);
		assert_eq!(res[0].len(), 5);
		Ok(())
	}

	#[tokio::test]
	async fn test_max_depth() -> anyhow::Result<()> {
		let _ = _Str.clone();

		let s = r#"{
        "key":"value"
        ,"complex_array": [
            {
                "number":1
                , "nest":[1,2]
                , "nest_obj": {"key":"value", "nest_arr":[1,2,3]}
            }
         ]
      }"#;
		let opt = ParserOptions::fmt().with_max_depth(1).with_sep("_");
		info!("opt==={:?}", opt);
		let p = ParserOptions::fmt().with_max_depth(1).with_sep("_").init();

		let res = p.run(s).await;
		match res {
			Ok(rr) => {
				// ...
				debug!("{:?}", rr);
			}
			Err(err) => {
				error!("parser error {:?}", err)
			}
		}
		Ok(())
	}
}
