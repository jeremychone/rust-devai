use crate::{Error, Result};
use rhai::{Array, Dynamic, Map, Scope};
use serde_json::{Map as SerdeMap, Value};

// region:    --- Dynamic Helpers

/// Make a Dynamic of type String or type Array of String, as Vec<String>
pub fn dynamic_into_strings(mut dynamic: Dynamic, err_suffix: &'static str) -> Result<Vec<String>> {
	let values: Vec<String> = if let Ok(single) = dynamic.as_immutable_string_ref() {
		// `take_immutable_string` consumes the value and gives an owned `String`
		vec![(*single).to_string()]
	} else if let Ok(arr) = dynamic.as_array_ref() {
		// Collect each item from the array and try to cast it as `String`
		arr.iter()
			.filter_map(|item| item.as_immutable_string_ref().ok().map(|v| (*v).to_string()))
			.collect::<Vec<_>>()
	} else {
		// Return an error if neither conversion worked
		return Err(Error::custom(format!(
			"'{err_suffix}' is not of type String or Array Of String"
		)));
	};

	Ok(values)
}

// endregion: --- Dynamic Helpers

// region:    --- Serde/Dynamic Helpers

pub fn dynamics_to_values(dynamics: Vec<Dynamic>) -> Result<Vec<Value>> {
	dynamics.into_iter().map(dynamic_to_value).collect::<Result<Vec<_>>>()
}

/// TODO: Need to look if the rhai serde feature provide that better
pub fn dynamic_to_value(dynamic: Dynamic) -> Result<Value> {
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
		let arr = dynamic.into_array()?;
		let serde_array = arr.into_iter().map(dynamic_to_value).collect::<Result<Vec<Value>>>()?;
		Value::Array(serde_array)
	} else if dynamic.is::<Map>() {
		let map = dynamic.cast::<rhai::Map>();
		let mut serde_map = SerdeMap::new();
		for (k, v) in map {
			serde_map.insert(k.to_string(), dynamic_to_value(v)?);
		}
		Value::Object(serde_map)
	} else {
		// If it's none of the above, return a null value
		Value::Null
	};

	Ok(val)
}

pub fn value_to_scope(value: &Value) -> Result<Scope> {
	let mut scope = Scope::new();

	match value {
		Value::Object(map) => {
			for (k, v) in map {
				let dynamic_value = value_to_dynamic(v);
				scope.push_dynamic(k.as_str(), dynamic_value);
			}
			Ok(scope)
		}
		_ => Err("Root value must be an object".into()),
	}
}

/// Here is a custom serde json to dynamic
/// TODO: Need to look if the rhai serde feature provide that better
pub fn value_to_dynamic(value: &Value) -> Dynamic {
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
				rhai_array.push(value_to_dynamic(v));
			}
			rhai_array.into()
		}
		Value::Object(obj) => {
			let mut rhai_map = Map::new();
			for (k, v) in obj {
				rhai_map.insert(k.clone().into(), value_to_dynamic(v));
			}
			rhai_map.into()
		}
	}
}

// endregion: --- Serde/Dynamic Helpers
