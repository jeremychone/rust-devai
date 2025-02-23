// region:    --- Modules

use crate::Result;
use handlebars::Handlebars;
use serde_json::Value;
use std::sync::{Arc, LazyLock};

// endregion: --- Modules

static HANDLEBARS: LazyLock<Arc<Handlebars>> = LazyLock::new(|| {
	let mut handlebars = Handlebars::new();
	// Disable escaping globally
	handlebars.register_escape_fn(|s| s.to_string());

	Arc::new(handlebars)
});

pub fn hbs_render(hbs_tmpl: &str, data_root: &Value) -> Result<String> {
	let handlebars = &*HANDLEBARS;
	let res = handlebars.render_template(hbs_tmpl, &data_root)?;
	Ok(res)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::assert_contains;
	use crate::run::Runtime;
	use crate::support::hbs::hbs_render;
	use serde_json::json;

	#[tokio::test]
	async fn test_hbs_with_lua_ok() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let script = r#"
        local file1 = utils.file.load("file-01.txt")
        local file2 = utils.file.load("agent-script/agent-before-all.aip")
        return {file1,file2}  -- Return an array of File structs
    "#;
		let tmpl = r#"
The files are: 
{{#each data}}
- {{this.path}}
{{/each}}
		"#;

		// -- Exec
		let lua_engine = runtime.new_lua_engine()?;
		let data = lua_engine.eval(script, None, None)?;
		let data = serde_json::to_value(data)?;
		let value = json!({
			"data": data
		});
		// Execute the template
		let res = hbs_render(tmpl, &value)?;

		// // -- Check
		assert_contains(&res, "- file-01.txt");
		assert_contains(&res, "- agent-script/agent-before-all.aip");

		Ok(())
	}
}

// endregion: --- Tests
