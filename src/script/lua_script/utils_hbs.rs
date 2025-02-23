use crate::Result;
use crate::run::RuntimeContext;
use mlua::{Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;
	table.set("render", lua.create_function(render)?)?;
	Ok(table)
}

/// ## Lua Documentation
/// ```lua
/// utils.hbs.render(content: string, data: any) -> string
/// ```
///
/// Renders a Handlebars template using the provided data.
/// Data can be any Lua value which will be converted to a serde_json::Value
/// using mlua's conversion mechanisms.
fn render(_lua: &Lua, (content, data): (String, Value)) -> mlua::Result<String> {
	let data_serde = serde_json::to_value(&data)
		.map_err(|err| crate::Error::custom(format!("Fail to convert lua value to serde. Cause: {err}")))?;
	let rendered = crate::support::hbs::hbs_render(&content, &data_serde).map_err(mlua::Error::external)?;
	Ok(rendered)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{eval_lua, setup_lua};

	#[tokio::test]
	async fn test_lua_hbs_render_simple() -> Result<()> {
		// Setup the Lua instance with the hbs module
		let lua = setup_lua(super::init_module, "hbs")?;

		// Lua script that calls the `utils.hbs.render` function with a simple template
		let lua_code = r#"
            local result = utils.hbs.render("Hello, {{name}}!", {name = "World"})
            return result
		"#;
		let res = eval_lua(&lua, lua_code)?;
		assert_eq!(res.as_str().ok_or("Result should be a string")?, "Hello, World!");
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_hbs_render_nested_table() -> Result<()> {
		// Setup the Lua instance with the hbs module
		let lua = setup_lua(super::init_module, "hbs")?;

		// Lua script that calls `utils.hbs.render` with a nested Lua table as data
		let lua_code = r#"
            local result = utils.hbs.render("ID: {{id}}, Nested: {{nested.value}}", {id = 42, nested = {value = "test"}})
            return result
		"#;
		let res = eval_lua(&lua, lua_code)?;
		assert_eq!(res.as_str().ok_or("Result should be a string")?, "ID: 42, Nested: test");
		Ok(())
	}
}

// endregion: --- Tests
