// region:    --- Modules

mod engine;
mod helpers;

use crate::Result;
use engine::rhai_engine;
use helpers::rhai_dynamic_to_serde_value;
use rhai::{Array, Dynamic, Scope};
use serde_json::Value;

// endregion: --- Modules

pub fn rhai_eval(script: &str, args: Option<&[&str]>) -> Result<Value> {
	// Initialize the Rhai engine
	let engine = rhai_engine()?;

	// Create a scope for variables
	let mut scope = Scope::new();

	// If args are provided, add them to the scope
	if let Some(args) = args {
		// Convert the list of &str to Rhai Dynamic Array
		let args_array: Array = args
			.iter()
			.map(|&arg| Dynamic::from(arg.to_string())) // Convert each &str to Dynamic
			.collect();

		// Push the args array into the scope under the name "args"
		scope.push_dynamic("args", Dynamic::from_array(args_array));
	}

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
	use rhai::Array;
	use value_ext::JsonValueExt;

	#[test]
	fn test_eval_load_file_ok() -> Result<()> {
		// -- Setup & Fixtures
		let engine = rhai_engine()?;
		let script = r#"
        let file1 = load_file("src/main.rs");
        let file2 = load_file("src/error.rs");
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
