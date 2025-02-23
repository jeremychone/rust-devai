//! Defines the `html` module, used in the lua engine
//!
//! ---
//!
//! ## Lua documentation
//! This module exposes functions that process HTML content.
//!
//! ### Functions
//! * `utils.html.prune_to_content(html_content: string) -> string`

use crate::Result;
use crate::run::RuntimeContext;
use crate::support::html::prune_to_content;
use mlua::{Lua, Table};

pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	let prune_fn = lua.create_function(prune_to_content_lua)?;
	table.set("prune_to_content", prune_fn)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Strips non-content elements from the provided HTML content and returns the cleaned HTML as a string.
///
/// This function removes:
/// - Non-visible tags such as `<script>`, `<link>`, `<style>`, and `<svg>`.
/// - HTML comments.
/// - Empty lines.
/// - Attributes except for `class`, `aria-label`, and `href`.
///
/// ```lua
/// local cleaned_html = utils.html.prune_to_content(html_content)
/// ```
///
fn prune_to_content_lua(_lua: &Lua, html_content: String) -> mlua::Result<String> {
	prune_to_content(html_content)
		.map_err(|err| mlua::Error::RuntimeError(format!("Failed to prune HTML content: {}", err)))
}

// region:    --- Tests
#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use crate::_test_support::{eval_lua, setup_lua};

	#[tokio::test]
	async fn test_lua_html_prune_to_content_ok() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::init_module, "html")?;
		let fx_script = r#"
local html_content = [[
<!DOCTYPE html>
<html>
<head>
    <script>alert('test');</script>
    <style>body { color: red; }</style>
</head>
<body>
    <div class="content">Hello World</div>
    <!-- comment -->
</body>
</html>
]]
return utils.html.prune_to_content(html_content)
        "#;
		// -- Exec
		let res = eval_lua(&lua, fx_script)?;
		// -- Check
		let cleaned_html = res.as_str().unwrap();
		assert!(!cleaned_html.contains("<script>"));
		assert!(!cleaned_html.contains("<style>"));
		assert!(!cleaned_html.contains("<!-- comment -->"));
		assert!(cleaned_html.contains(r#"<div class="content">Hello World</div>"#));
		Ok(())
	}
}
// endregion: --- Tests
