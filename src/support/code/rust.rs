use crate::Result;
use logos::Logos;

/// Trims Rust code to keep only function declarations by replacing function bodies with `{ ... }`.
/// Preserves comments, whitespace, and non-function code structures.
///
/// # Example
///
/// ```
/// let code = r#"
/// // A simple function
/// fn add(a: i32, b: i32) -> i32 {
///     a + b
/// }
///
/// fn main() {
///     let result = add(1, 2);
///     println!("{}", result);
/// }
/// "#;
///
/// let result = rust_trim_to_declaration(code)?;
/// assert_eq!(result, r#"
/// // A simple function
/// fn add(a: i32, b: i32) -> i32 {
///     // ...
/// }
///
/// fn main() {
///     // ...
/// }
/// "#);
/// ```
pub fn rust_trim_to_declaration(code: &str) -> Result<String> {
	let mut lexer = Token::lexer(code);
	let mut result: Vec<&str> = Vec::with_capacity(32);
	let mut brace_count = 0;
	let mut in_function = false;

	while let Some(token) = lexer.next() {
		match token.map_err(|_| "lexer next error ()")? {
			Token::Comment => {
				if !in_function || brace_count == 0 {
					result.push(lexer.slice());
				}
			}
			Token::Fn => {
				result.push(lexer.slice());
				in_function = true;
			}
			Token::OpenBrace => {
				if in_function {
					brace_count += 1;
					if brace_count == 1 {
						result.push(" {\n    // ...\n}\n");
					}
				} else {
					result.push("{");
				}
			}
			Token::CloseBrace => {
				if in_function && brace_count > 0 {
					brace_count -= 1;
					if brace_count == 0 {
						in_function = false;
					}
				} else {
					result.push("}");
				}
			}
			Token::Text => {
				if !in_function || brace_count == 0 {
					result.push(lexer.slice());
				}
			}
			Token::Newline => {
				if !in_function || brace_count == 0 {
					result.push("\n");
				}
			}
			Token::Whitespace => {
				if !in_function || brace_count == 0 {
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

	#[regex(r"fn ", priority = 2)]
	Fn,

	// #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*|[^{}\n\r\sa-zA-Z]+", priority = 1)]
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
