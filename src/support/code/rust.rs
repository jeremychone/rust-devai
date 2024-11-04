use crate::Result;
use logos::Logos;

/// Trims Rust code to keep only function declarations by:
///
/// - Replacing function bodies with `{ ... }`.
/// - Redacting the contents of `#[cfg(test)]` modules.
/// - Removing the contents of test functions and blocks.
///
/// Preserves comments, whitespace, and non-function code structures.
///
/// # Example
///
/// ```
/// let code = r#"
/// // A simple function
/// fn add(a: i32, b: i32) -> i32 {
///     a + b;
/// }
///
/// #[cfg(test)]
/// mod tests {
///     #[test]
///     fn test_add() {
///         assert_eq!(add(1, 2), 3);
///     }
/// }
/// "#;
///
/// let result = run_prune_to_declarations(code)?;
/// assert_eq!(result, r#"
/// // A simple function
/// fn add(a: i32, b: i32) -> i32 {
///     // ...
/// }
///
/// #[cfg(test)]
/// mod tests {
///     // ...
/// }
/// "#);
/// ```
pub fn run_prune_to_declarations(code: &str) -> Result<String> {
	let mut lexer = Token::lexer(code);
	let mut result: Vec<&str> = Vec::with_capacity(32);
	let mut brace_count = 0;
	let mut in_fn = false;
	let mut after_cfg_test = false;
	let mut in_test_block = false;
	#[allow(unused)] // not sure why it said unused
	let mut should_capture = true;

	while let Some(token) = lexer.next() {
		should_capture = !in_test_block && !in_fn;

		match token.map_err(|_| "lexer next error ()")? {
			Token::Comment => {
				if should_capture || brace_count == 0 {
					result.push(lexer.slice());
				}
			}
			Token::Fn => {
				if should_capture {
					result.push(lexer.slice());
				}
				in_fn = true;
			}
			Token::CfgTest => {
				result.push(lexer.slice());
				after_cfg_test = true;
			}

			Token::OpenBrace => {
				if after_cfg_test && brace_count == 0 {
					in_test_block = true;
				}

				if in_fn || in_test_block {
					brace_count += 1;
					if brace_count == 1 {
						result.push(" {\n    // ...\n}\n");
					}
				} else {
					result.push("{");
				}
			}
			Token::CloseBrace => {
				if (in_fn || in_test_block) && brace_count > 0 {
					brace_count -= 1;
					if brace_count == 0 {
						if in_fn {
							in_fn = false;
						}
						if in_test_block {
							after_cfg_test = false;
							in_test_block = false;
						}
					}
				} else {
					result.push("}");
				}
			}
			Token::Text => {
				if should_capture || brace_count == 0 {
					result.push(lexer.slice());
				}
			}
			Token::Newline => {
				if should_capture || brace_count == 0 {
					result.push("\n");
				}
			}
			Token::Whitespace => {
				if should_capture || brace_count == 0 {
					result.push(lexer.slice());
				}
			}
		}
	}

	Ok(result.join(""))
}

#[derive(Logos, Debug, PartialEq)]
enum Token {
	#[regex(r"//.*", priority = 3)]
	Comment,

	#[regex(r"#\[cfg\(test", priority = 2)]
	CfgTest,

	#[regex(r"fn ", priority = 2)]
	Fn,

	#[regex(r"[a-zA-Z_][a-zA-Z0-9_]*|[^{}\n\r\sa-zA-Z]+", priority = 1)]
	Text,

	#[token("{")]
	OpenBrace,

	#[token("}")]
	CloseBrace,

	#[regex(r"[\n\r]")]
	Newline,

	#[regex(r"[ \t]+")]
	Whitespace,
}

// region:    --- Tests

#[cfg(test)]
mod tests {

	use super::*;

	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	#[tokio::test]
	async fn test_rust_prune_to_declarations() -> Result<()> {
		// -- Fixtures
		let data_script = r#"
//! Some top comment 

use some::module; // and comment 

/// Some comment
/// Some stuff about fn 
pub fn async some_async_fn(some_arg: String) -> i32{
   let some = "code";
	 123
}

// Some fn normal
fn some_normal() {
		// DOING SOME STUFF
		// some fn stuff
}	 

#[cfg(test)]
mod tests {
 // Some tests
 #[test]
 fn some_test() { 
 // some test fn impl
 }
}
		"#;

		// -- Exec
		let res = run_prune_to_declarations(data_script)?;

		println!("->> \n{res}");

		// -- Check
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
		assert!(
			!res.contains("// DOING SOME STUFF"),
			"should not have //DOING SOME STUFF"
		);
		assert!(!res.contains("// Some tests"), "should not have // Some tests");

		Ok(())
	}
}

// endregion: --- Tests
