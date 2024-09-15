use crate::Result;
use rhai::{Array, Dynamic, Map};
use serde_json::{json, Map as SerdeMap, Value};
use std::collections::HashMap;

pub fn rhai_dynamic_to_serde_value(dynamic: Dynamic) -> Result<Value> {
	// Check the type of Dynamic and convert it to serde_json::Value
	let val = if dynamic.is::<i64>() {
		Value::Number(dynamic.as_int()?.into())
	} else if dynamic.is::<f64>() {
		Value::Number(serde_json::Number::from_f64(dynamic.as_float()?).ok_or("not a json number")?)
	} else if dynamic.is::<bool>() {
		Value::Bool(dynamic.as_bool()?)
	} else if dynamic.is::<String>() {
		Value::String(dynamic.into_string()?)
	} else if dynamic.is::<Array>() {
		let arr = dynamic.into_array().unwrap();
		let serde_array = arr
			.into_iter()
			.map(rhai_dynamic_to_serde_value)
			.collect::<Result<Vec<Value>>>()?;
		Value::Array(serde_array)
	} else if dynamic.is::<Map>() {
		let map = dynamic.cast::<rhai::Map>();
		let mut serde_map = SerdeMap::new();
		for (k, v) in map {
			serde_map.insert(k.to_string(), rhai_dynamic_to_serde_value(v)?);
		}
		Value::Object(serde_map)
	} else {
		// If it's none of the above, return a null value
		Value::Null
	};

	Ok(val)
}
