// region:    --- Modules

use crate::Result;
use handlebars::Handlebars;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

// endregion: --- Modules

static HANDLEBARS: LazyLock<Arc<Handlebars>> = LazyLock::new(|| {
	let mut handlebars = Handlebars::new();
	// Disable escaping globally
	handlebars.register_escape_fn(|s| s.to_string());

	Arc::new(handlebars)
});

pub fn hbs_render(hbs_tmpl: &str, data_root: &HashMap<String, Value>) -> Result<String> {
	let handlebars = &*HANDLEBARS;
	let res = handlebars.render_template(hbs_tmpl, &data_root)?;
	Ok(res)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Error = Box<dyn std::error::Error>;
	type Result<T> = core::result::Result<T, Error>; // For tests.

	use super::*;
	use crate::_test_support::assert_contains;
	use crate::run::Runtime;
	use crate::script::rhai_eval;

	#[tokio::test]
	async fn test_hbs_with_rhai_ok() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let script = r#"
        let file1 = file::load("file-01.txt");
        let file2 = file::load("agent-script/agent-before-all.devai");
        [file1, file2]  // Return an array of File structs
    "#;
		let tmpl = r#"
The files are: 
{{#each data}}
- {{this.path}}
{{/each}}
		"#;

		// -- Exec
		let result_json = rhai_eval(runtime.rhai_engine(), script, None)?;
		// Execute the template
		let data = HashMap::from([("data".to_string(), result_json)]);
		let res = hbs_render(tmpl, &data)?;

		// -- Check
		assert_contains(&res, "- file-01.txt");
		assert_contains(&res, "- agent-script/agent-before-all.devai");

		Ok(())
	}
}

// endregion: --- Tests
