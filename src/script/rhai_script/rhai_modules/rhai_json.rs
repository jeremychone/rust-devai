//! Defines the `json` module, used in the rhai engine.
//!
//! ---
//!
//! ## RHAI documentation
//! The `json` module exposes functions to parse and stringify JSON content.
//!
//! ### Functions
//! * `json::parse(content: string) -> dynamic`
//! * `json::stringify(content: dynamic) -> string`

use crate::script::value_to_dynamic;
use crate::Error;
use rhai::plugin::*;
use rhai::{Dynamic, FuncRegistration, Module};
use serde_json::Value;

pub fn rhai_module() -> Module {
	let mut module = Module::new();

	FuncRegistration::new("parse")
		.in_global_namespace()
		.set_into_module(&mut module, parse);

	FuncRegistration::new("stringify")
		.in_global_namespace()
		.set_into_module(&mut module, stringify);

	FuncRegistration::new("stringify_to_line")
		.in_global_namespace()
		.set_into_module(&mut module, stringify_to_line);

	module
}

/// ## RHAI Documentation
///
/// Parse a JSON string into a dynamic object.
///
/// ```
/// // API Signature
/// json::parse(content: string) -> dynamic (throws: JsonParseException)
/// ```
///
/// Parse a JSON string into a dynamic object that can be used in the RHAI script.
///
/// ### Example
/// ```
/// let json_str = "{\"name\": \"John\", \"age\": 30}";
/// let obj = json::parse(json_str);
/// print(obj.name); // prints "John"
/// ```
///
/// ### Returns
///
/// Returns a dynamic object representing the parsed JSON.
///
/// ### Exception (JsonParseException)
///
/// ```
/// {
///   error: string  // Error message from JSON parsing
/// }
/// ```
fn parse(content: &str) -> RhaiResult {
	match serde_json::from_str::<Value>(content) {
		Ok(val) => Ok(value_to_dynamic(val)),
		Err(err) => Err(Error::cc("json::parse failed.", err).into()),
	}
}

/// ## RHAI Documentation
///
/// Stringify a dynamic object into a JSON string with pretty formatting.
///
/// ```
/// // API Signature
/// json::stringify(content: dynamic) -> string (throws: JsonStringifyException)
/// ```
///
/// Convert a dynamic object into a JSON string with pretty formatting using tab indentation.
///
/// ### Example
/// ```
/// let obj = #{
///     name: "John",
///     age: 30
/// };
/// let json_str = json::stringify(obj);
/// // Result will be:
/// // {
/// //     "name": "John",
/// //     "age": 30
/// // }
/// ```
///
/// ### Returns
///
/// Returns a formatted JSON string.
///
/// ### Exception (JsonStringifyException)
///
/// ```
/// {
///   error: string  // Error message from JSON stringification
/// }
/// ```
fn stringify(content: Dynamic) -> RhaiResult {
	match serde_json::to_value(content) {
		Ok(val) => match serde_json::to_string_pretty(&val) {
			Ok(str) => Ok(Dynamic::from(str)),
			Err(err) => Err(Error::cc("Fail to stringify", err).into()),
		},
		Err(err) => Err(Error::cc("Fail to to_value the dynamic", err).into()),
	}
}

/// ## RHAI Documentation
///
/// Stringify a dynamic object into a single line.
///
/// Good for newline json
///
/// ```
/// // API Signature
/// json::stringify_to_line(content: dynamic) -> string (throws: JsonStringifyException)
/// ```
///
/// Convert a dynamic object into a JSON string with pretty formatting using tab indentation.
///
/// ### Example
/// ```
/// let obj = #{
///     name: "John",
///     age: 30
/// };
/// let json_str = json::stringify(obj);
/// // Result will be:
/// // {"name": "John","age": 30}
/// ```
///
/// ### Returns
///
/// Returns a formatted JSON string.
///
/// ### Exception (JsonStringifyException)
///
/// ```
/// {
///   error: string  // Error message from JSON stringification
/// }
/// ```
fn stringify_to_line(content: Dynamic) -> RhaiResult {
	match serde_json::to_value(content) {
		Ok(val) => match serde_json::to_string(&val) {
			Ok(str) => Ok(Dynamic::from(str)),
			Err(err) => Err(Error::cc("Fail to stringify", err).into()),
		},
		Err(err) => Err(Error::cc("Fail to to_value the dynamic", err).into()),
	}
}

#[cfg(test)]
mod tests {
	use crate::_test_support::{assert_contains, assert_not_contains, run_reflective_agent};
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_rhai_json_parse() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            let content = "{\"name\": \"John\", \"age\": 30}";
            return json::parse(content);
        "#;

		let res = run_reflective_agent(script, None).await?;

		assert_eq!(res.x_get_str("name")?, "John");
		assert_eq!(res.x_get_i64("age")?, 30);

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_json_parse_invalid() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            try {
                let content = "{invalid_json}";
                json::parse(content);
                return "should not reach here";
            } catch(ex) {
                return ex;
            }
        "#;

		// -- Exec
		let res = run_reflective_agent(script, None).await?;

		// -- Check
		assert!(res.x_get_str("error")?.contains("key must be a string "));

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_json_stringify() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            let obj = #{
                name: "John",
                age: 30
            };
            return json::stringify(obj);
        "#;

		let res = run_reflective_agent(script, None).await?;
		let result = res.as_str().unwrap();

		// Parse the result back to verify it's valid JSON
		let parsed: serde_json::Value = serde_json::from_str(result)?;
		assert_eq!(parsed["name"], "John");
		assert_eq!(parsed["age"], 30);
		assert!(result.contains("\n")); // Verify it's pretty printed
		assert!(result.contains("  ")); // Verify it has indentation

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_json_stringify_complex() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            let obj = #{
                name: "John",
                age: 30,
                address: #{
                    street: "123 Main St",
                    city: "New York"
                },
                hobbies: ["reading", "gaming"]
            };
            return json::stringify(obj);
        "#;

		// -- Exec
		let res = run_reflective_agent(script, None).await?;

		// -- Check
		let result = res.as_str().ok_or("should be string")?;

		// Parse the result back to verify it's valid JSON
		let parsed: serde_json::Value = serde_json::from_str(result)?;
		assert_eq!(parsed["name"], "John");
		assert_eq!(parsed["age"], 30);
		assert_eq!(parsed["address"]["street"], "123 Main St");
		assert_eq!(parsed["hobbies"][0], "reading");
		assert!(result.contains("\n")); // Verify it's pretty printed
		assert!(result.contains("  ")); // Verify it has indentation

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_json_stringify_to_line() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            let obj = #{
                name: "John",
                age: 30,
                address: #{
                    street: "123 Main St",
                    city: "New York"
                },
                hobbies: ["reading", "gaming"]
            };
            return json::stringify_to_line(obj);
        "#;

		// -- Exec
		let res = run_reflective_agent(script, None).await?;

		// -- Check
		let res = res.as_str().ok_or("should be string")?;
		assert_contains(res, r#""name":"John""#);
		assert_not_contains(res, "\n");
		assert_not_contains(res, "  ");

		Ok(())
	}
}
