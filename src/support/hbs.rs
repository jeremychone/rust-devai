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
	use crate::script::rhai_eval;

	#[test]
	fn test_hbs_with_rhai_ok() -> Result<()> {
		// -- Setup & Fixtures
		let script = r#"
        let file1 = file::load("src/main.rs");
        let file2 = file::load("src/error.rs");
        [file1, file2]  // Return an array of File structs
    "#;
		let tmpl = r#"
The files are: 
{{#each data}}
- {{this.path}}
{{/each}}
		"#;

		// -- Exec
		let result_json = rhai_eval(script, None, None)?;
		// Execute the template
		let data = HashMap::from([("data".to_string(), result_json)]);
		let res = hbs_render(tmpl, &data)?;

		// -- Check
		assert!(res.contains("- src/main.rs"), "- src/main.rs");
		assert!(res.contains("- src/error.rs"), "- src/error.rs");

		Ok(())
	}
}

// endregion: --- Tests
