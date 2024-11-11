//! Defines the `rust` module, used in the rhai engine.
//!
//! ---
//!
//! ## RHAI documentation
//! The `rust` module exposes functions used to process Rust code.
//!
//! ### Functions
//! * `rust::prune_to_declarations(code: string) -> string`

use crate::support::code::run_prune_to_declarations;
use rhai::plugin::RhaiResult;
use rhai::{EvalAltResult, FuncRegistration, Module};

pub fn rhai_module() -> Module {
	let mut module = Module::new();

	FuncRegistration::new("prune_to_declarations")
		.in_global_namespace()
		.set_into_module(&mut module, prune_to_declarations);

	module
}

// region:    --- Rhai Functions

/// ## RHAI Documentation
/// ```rhai
/// rust::prune_to_declarations(code: string) -> string
/// ```
///
/// Trims Rust code to keep only function declarations by replacing function bodies with `{ ... }`.
/// Preserves comments, whitespace, and non-function code structures.
///
/// Example:
/// ```rhai
/// let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
/// let result = rust::prune_to_declarations(code);
/// // result will be: "fn add(a: i32, b: i32) -> i32 { ... }"
/// ```
fn prune_to_declarations(code: &str) -> RhaiResult {
	match run_prune_to_declarations(code) {
		Ok(result) => Ok(result.into()),
		Err(err) => Err(Box::new(EvalAltResult::ErrorRuntime(
			format!("Failed to prune Rust code: {}", err).into(),
			rhai::Position::NONE,
		))),
	}
}

// endregion: --- Rhai Functions

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::run_reflective_agent;
	use serde_json::Value;

	#[tokio::test]
	async fn test_rhai_rust_prune_to_declarations() -> Result<()> {
		// -- Fixtures
		let data_script = r#"
//! Some top comment 

use some::module; // and comment 

/// Some comment
pub fn async some_async_fn(some_arg: String) -> i32{
   let some = "code";
	 123
}

// Some fn normal
fn some_normal() {
		// DOING SOME STUFF
		// some fn stuff
}	 
		"#;

		// -- Exec
		let res = run_reflective_agent(
			r#"return rust::prune_to_declarations(input);"#,
			Some(Value::String(data_script.to_string())),
		)
		.await?;

		// -- Check
		let res = res.as_str().ok_or("Should be str")?;
		assert!(
			res.contains("use some::module; // and comment "),
			"should contain use ..."
		);
		assert!(
			res.contains("async some_async_fn(some_arg: String) -> i32"),
			"should contain some_async_fn"
		);
		assert!(res.contains("fn some_normal()"), "should contain some_normal");
		assert!(
			!res.contains(r#"let some = "code";"#),
			"should NOT contain let some ..."
		);
		assert!(!res.contains("// DOING SOME STUFF"), "DOING SOME STUFF");

		Ok(())
	}
}

// endregion: --- Tests
