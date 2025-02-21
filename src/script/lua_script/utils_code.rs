//! Defines the `code` module, used in the lua engine.
//!
//! ### Lua Documentation
//! This module exposes functions that process code formatting.
//!
//! #### Functions
//! * `utils.code.comment_line(lang_ext: string, comment_content: string) -> string`
//!    - Creates a single line comment in the appropriate style based on the language extension.
//!    - Supported extensions:
//!       - "lua", "sql": uses `-- ...`
//!       - "html": uses `<!-- ... -->`
//!       - "css", "pcss": uses `/* ... */`
//!       - "py": uses `# ...`
//!       - Fallback: uses `// ...`
//!
//! The returned string does not include a trailing newline.

use crate::Result;
use mlua::{Lua, Table};

pub fn init_module(lua: &Lua, _runtime_context: &crate::run::RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	table.set("comment_line", lua.create_function(comment_line)?)?;

	Ok(table)
}

/// Creates a comment line based on the language extension and the given comment content.
///
/// # Arguments
///
/// * `lang_ext` - A string slice representing the file extension or language identifier (e.g., "rs", "lua", "py").
/// * `comment_content` - The content that should be commented.
///
/// # Returns
///
/// A string representing the commented line without a trailing newline.
///
/// # Examples
///
/// For example, in Lua:
/// ```lua
/// local comment = utils.code.comment_line("rs", "This is a rust comment")
/// -- comment will be: "// This is a rust comment"
/// ```
fn comment_line(_lua: &Lua, (lang_ext, comment_content): (String, String)) -> mlua::Result<String> {
	// Normalize the language extension by trimming and converting to lowercase.
	let ext = lang_ext.trim().to_lowercase();
	let comment = match ext.as_str() {
		"lua" | "sql" => format!("-- {}", comment_content),
		"html" => format!("<!-- {} -->", comment_content),
		"css" | "pcss" => format!("/* {} */", comment_content),
		"py" => format!("# {}", comment_content),
		_ => format!("// {}", comment_content),
	};
	Ok(comment)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{eval_lua, setup_lua};

	#[test]
	fn test_code_comment_line_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::init_module, "code")?;
		// Define test cases as tuples: (language extension, comment content, expected result)
		let test_cases = [
			("rs", "Rust comment", "// Rust comment"),
			("lua", "Lua comment", "-- Lua comment"),
			("sql", "SQL comment", "-- SQL comment"),
			("html", "HTML comment", "<!-- HTML comment -->"),
			("css", "CSS comment", "/* CSS comment */"),
			("pcss", "PCSS comment", "/* PCSS comment */"),
			("py", "Python comment", "# Python comment"),
			("js", "JavaScript comment", "// JavaScript comment"),
		];

		// -- Exec & Check
		for (lang, content, expected) in test_cases.iter() {
			let script = format!("return utils.code.comment_line({:?}, {:?})", lang, content);
			let res = eval_lua(&lua, &script)?;
			let res_str = res.as_str().ok_or("Expected a string result")?;
			assert_eq!(
				res_str, *expected,
				"Failed for lang_ext: {} with content: {}",
				lang, content
			);
		}
		Ok(())
	}
}

// endregion: --- Tests
