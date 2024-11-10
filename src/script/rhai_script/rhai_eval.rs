use super::dynamic_helpers::{dynamic_to_value, value_to_scope};
use crate::Result;
use rhai::{Dynamic, Engine, Scope};
use serde_json::Value;

pub fn rhai_eval(engine: &Engine, script: &str, scope_value: Option<Value>) -> Result<Value> {
	// Initialize the Rhai engine

	// Create a scope for variables
	let mut scope = if let Some(scope_value) = scope_value.as_ref() {
		value_to_scope(scope_value)?
	} else {
		Scope::new()
	};

	// Evaluate the script with the provided scope

	let result = engine.eval_with_scope::<Dynamic>(&mut scope, script)?;

	// Convert the result to a serde_json::Value
	let result_json = dynamic_to_value(result)?;
	Ok(result_json)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Error = Box<dyn std::error::Error>;
	type Result<T> = core::result::Result<T, Error>; // For tests.

	use crate::run::Runtime;

	/// Lower engine-level eval test
	#[tokio::test]
	async fn test_engine_eval_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let script_content = r#"
        let x = 10;
        let y = 20;
        x + y
    "#;

		// -- Exec
		let result = runtime.rhai_engine().eval::<i64>(script_content)?;

		// -- Check
		assert_eq!(result, 30);

		Ok(())
	}
}

// endregion: --- Tests
