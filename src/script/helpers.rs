use crate::Result;
use rhai::{Array, Dynamic, Map, Scope};
use serde_json::{Map as SerdeMap, Value};

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

pub fn serde_value_to_scope(value: &Value) -> Result<Scope> {
	let mut scope = Scope::new();

	match value {
		Value::Object(map) => {
			for (k, v) in map {
				let dynamic_value = serde_value_to_dynamic(v);
				scope.push_dynamic(k.as_str(), dynamic_value);
			}
			Ok(scope)
		}
		_ => Err("Root value must be an object".into()),
	}
}

pub fn serde_value_to_dynamic(value: &Value) -> Dynamic {
	match value {
		Value::Null => Dynamic::UNIT,
		Value::Bool(b) => (*b).into(),
		Value::Number(n) => {
			if let Some(i) = n.as_i64() {
				i.into()
			} else if let Some(f) = n.as_f64() {
				f.into()
			} else {
				Dynamic::UNIT
			}
		}
		Value::String(s) => s.clone().into(),
		Value::Array(arr) => {
			let mut rhai_array = Array::new();
			for v in arr {
				rhai_array.push(serde_value_to_dynamic(v));
			}
			rhai_array.into()
		}
		Value::Object(obj) => {
			let mut rhai_map = Map::new();
			for (k, v) in obj {
				rhai_map.insert(k.clone().into(), serde_value_to_dynamic(v));
			}
			rhai_map.into()
		}
	}
}
