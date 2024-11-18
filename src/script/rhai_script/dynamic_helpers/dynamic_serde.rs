use crate::{Error, Result};
use rhai::{Array, Dynamic, Map, Scope};
use serde_json::{Map as SerdeMap, Value};

// region:    --- Dynamic Helpers

/// Make a Dynamic of type String or type Array of String, as `Vec<String>`
pub fn dynamic_into_strings(dynamic: Dynamic, err_suffix: &'static str) -> Result<Vec<String>> {
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

/// Make a list of dynamics into a list of serde value
pub fn dynamics_to_values(dynamics: Vec<Dynamic>) -> Result<Vec<Value>> {
	dynamics.into_iter().map(dynamic_to_value).collect::<Result<Vec<_>>>()
}

/// Just a passthrough to Rhai serde implementation,
/// with a custom map error.
pub fn dynamic_to_value(dynamic: Dynamic) -> Result<Value> {
	let val = serde_json::to_value(dynamic).map_err(Error::RhaiDynamicToValue)?;
	Ok(val)
}

/// All the rhai default deserialization,
/// but return Dynamic::UNIT if fails (for now)
pub fn value_to_dynamic(value: Value) -> Dynamic {
	serde_json::from_value(value).unwrap_or_else(|_| Dynamic::from(Dynamic::UNIT))
}

/// Create a Rhai scope from a Value.
/// Requires the Value is an Object.
pub fn value_to_scope(value: &Value) -> Result<Scope> {
	let mut scope = Scope::new();

	match value {
		Value::Object(map) => {
			for (k, v) in map {
				let dynamic_value = value_to_dynamic(v.clone());
				scope.push_dynamic(k.as_str(), dynamic_value);
			}
			Ok(scope)
		}
		_ => Err("Root value must be an object".into()),
	}
}

// endregion: --- Serde/Dynamic Helpers

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::script::rhai_script::dynamic_helpers::into_dynamic;
	use crate::script::{DynaMap, IntoDynamic as _};
	use serde_json::json;
	use value_ext::JsonValueExt;

	#[test]
	fn test_dynamic_helpers_to_dynamic_from_serde_object_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_value_1 = json!({
			"one": 1,
			"two": "some for two",
			"nest": {
				"three": [4, 5, "six"],
				"seven": null
			},
			"not_here": null
		});

		// -- Exec
		let dynamic: Dynamic = value_to_dynamic(fx_value_1);

		// -- Check
		let mut dyna = DynaMap::try_from(dynamic)?;
		let nest_dyna = dyna.remove_to_dynamic("nest")?.ok_or("missing nest")?;
		let nest_dyna = DynaMap::from_dynamic(nest_dyna)?;

		assert_eq!(dyna.get::<i64>("one")?, 1);
		assert!(
			nest_dyna.get::<Dynamic>("seven")?.is_unit(),
			"nest.seven should be UNIT"
		);

		Ok(())
	}

	#[test]
	fn test_dynamic_helpers_to_dynamic_from_serde_scalars_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_value_str = json!("some string");
		let fx_value_num = json!(123);
		let fx_value_bool = json!(false);

		// -- Exec
		let dyna_str: Dynamic = value_to_dynamic(fx_value_str);
		let fx_value_num: Dynamic = value_to_dynamic(fx_value_num);
		let fx_value_bool: Dynamic = value_to_dynamic(fx_value_bool);

		// -- Check
		assert_eq!(dyna_str.try_cast::<String>().ok_or("should be str")?, "some string");
		assert_eq!(fx_value_num.try_cast::<i64>().ok_or("should be i64")?, 123);
		assert!(
			!fx_value_bool.try_cast::<bool>().ok_or("should be bool")?,
			"fx_value_bool should be false"
		);

		Ok(())
	}

	#[test]
	fn test_dynamic_helpers_to_serde_from_dynamic_object_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_value = json!({
		  "one": 1,
		  "three": {
			"five": 5,
			"four": 4,
			"seven": null,
			"six": "some for 6"
		  },
		  "two": "some for two"
		});
		let dyna = DynaMap::default()
			.insert("one", 1)
			.insert("two", "some for two")
			.insert(
				"three",
				DynaMap::default()
					.insert("four", 4)
					.insert("five", 5)
					.insert("six", "some for 6")
					.insert("seven", ()),
			)
			.into_dynamic();

		// -- Exec
		let value: Value = dynamic_to_value(dyna)?;

		// -- Check
		assert_eq!(value, fx_value);

		Ok(())
	}

	#[test]
	fn test_dynamic_helpers_to_serde_from_dynamic_scalars_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_value_str = json!("some string");
		let fx_value_num = json!(123);
		let fx_value_bool = json!(false);

		let dyna_str = "some string".into_dynamic();
		let dyna_num = 123.into_dynamic();
		let dyna_bool = false.into_dynamic();

		// -- Exec
		let value_str: Value = dynamic_to_value(dyna_str)?;
		let value_num: Value = dynamic_to_value(dyna_num)?;
		let value_bool: Value = dynamic_to_value(dyna_bool)?;

		// -- Check
		assert_eq!(value_str, fx_value_str);
		assert_eq!(value_num, fx_value_num);
		assert_eq!(value_bool, fx_value_bool);

		Ok(())
	}
}

// endregion: --- Tests
