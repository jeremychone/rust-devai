//! Defines the `json` module, used in the rhai engine.
//!
//! ---
//!
//! ## RHAI documentation
//! The `json` module exposes functions to parse JSON content.
//!
//! ### Functions
//! * `json::parse(content: string) -> dynamic`

use crate::script::value_to_dynamic;
use crate::Error;
use rhai::plugin::RhaiResult;
use rhai::{FuncRegistration, Module};
use serde_json::Value;

pub fn rhai_module() -> Module {
	let mut module = Module::new();

	FuncRegistration::new("parse")
		.in_global_namespace()
		.set_into_module(&mut module, parse);

	module
}

/// ## RHAI Documentation
///
/// Parse a JSON string into a dynamic value.
///
/// ```
/// // API Signature
/// json::parse(content: string) -> dynamic (throws: JsonParseException)
/// ```
///
/// Parses a JSON string and returns a dynamic value that can be used in rhai.
///
/// ### Example
/// ```
/// let json_str = "{\"name\": \"John\", \"age\": 30}";
/// let data = json::parse(json_str);
/// print(data.name); // prints "John"
/// ```
///
/// ### Returns
///
/// Returns a dynamic value representing the parsed JSON structure.
///
/// ### Exception (JsonParseException)
///
/// Throws an error if the JSON string is invalid or cannot be parsed.
///
fn parse(content: &str) -> RhaiResult {
	match serde_json::from_str::<Value>(content) {
		Ok(value) => Ok(value_to_dynamic(value)),
		Err(err) => Err(Error::cc("JSON parse error", err).into()),
	}
}

#[cfg(test)]
mod tests {
	use crate::_test_support::run_reflective_agent;

	#[tokio::test]
	async fn test_rhai_json_parse_object() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            let data = json::parse("{\"name\": \"John\", \"age\": 30}");
            return data;
        "#;

		let res = run_reflective_agent(script, None).await?;
		assert_eq!(res["name"], "John");
		assert_eq!(res["age"], 30);

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_json_parse_array() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            let data = json::parse("[1, 2, 3]");
            return data;
        "#;

		let res = run_reflective_agent(script, None).await?;
		assert_eq!(res[0], 1);
		assert_eq!(res[1], 2);
		assert_eq!(res[2], 3);

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_json_parse_invalid() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            try {
                json::parse("invalid json");
                return "should not reach here";
            } catch(ex) {
                return ex;
            }
        "#;

		let res = run_reflective_agent(script, None).await?;
		assert!(res.to_string().contains("JSON parse error"));

		Ok(())
	}
}
