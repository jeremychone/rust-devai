use super::engine::rhai_engine;
use super::helpers::rhai_dynamic_to_serde_value;
use crate::script::helpers::{serde_value_to_dynamic, serde_value_to_scope};
use crate::Result;
use rhai::{Array, Dynamic, Scope};
use serde_json::Value;
use value_ext::JsonValueExt;

pub fn rhai_eval(script: &str, scope_value: Option<Value>) -> Result<Value> {
	// Initialize the Rhai engine
	let engine = rhai_engine()?;

	// Create a scope for variables
	let mut scope = if let Some(scope_value) = scope_value.as_ref() {
		serde_value_to_scope(scope_value)?
	} else {
		Scope::new()
	};

	// Evaluate the script with the provided scope
	let result = engine.eval_with_scope::<Dynamic>(&mut scope, script)?;

	// Convert the result to a serde_json::Value
	let result_json = rhai_dynamic_to_serde_value(result)?;

	Ok(result_json)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Error = Box<dyn std::error::Error>;
	type Result<T> = core::result::Result<T, Error>; // For tests.

	use super::*;
	use crate::script::engine::rhai_engine;
	use rhai::Array;
	use value_ext::JsonValueExt;

	#[test]
	fn test_eval_file_load_ok() -> Result<()> {
		// -- Setup & Fixtures
		let engine = rhai_engine()?;
		let script = r#"
        let file1 = file_load("src/main.rs");
        let file2 = file_load("src/error.rs");
        [file1, file2]  // Return an array of File structs
    "#;

		// -- Exec
		let result = rhai_eval(script, None)?;

		// -- Check
		if let Value::Array(values) = result {
			let zipped = values.iter().zip(["src/main.rs", "src/error.rs"].iter());

			for (val, expected_path) in zipped {
				let val_path = val.x_get::<String>("path")?;
				assert_eq!(expected_path, &val_path);
			}
		}

		Ok(())
	}

	/// Lower engine level eval test
	#[test]
	fn test_engine_eval_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let engine = rhai_engine()?;
		let script_content = r#"
        let x = 10;
        let y = 20;
        x + y
    "#;

		// -- Exec
		let result = engine.eval::<i64>(script_content)?;

		// -- Check
		assert_eq!(result, 30);

		Ok(())
	}
}

// endregion: --- Tests
