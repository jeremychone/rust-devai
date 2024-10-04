use crate::script::rhai_script::helpers::serde_value_to_dynamic;
use rhai::plugin::RhaiResult;
use rhai::{FuncRegistration, Module};
use serde_json::json;

pub fn rhai_module() -> Module {
	// Create a module for text functions
	let mut module = Module::new();

	FuncRegistration::new("action_skip")
		.in_global_namespace()
		.set_into_module(&mut module, action_skip);

	FuncRegistration::new("action_skip")
		.in_global_namespace()
		.set_into_module(&mut module, action_skip_with_reason);

	module
}

// region:    --- Rhai Functions

fn action_skip() -> RhaiResult {
	let res = json!({
		"_devai_": {
			"action": "skip"
		}
	});
	let res = serde_value_to_dynamic(&res);

	Ok(res)
}

fn action_skip_with_reason(reason: &str) -> RhaiResult {
	let res = json!({
		"_devai_": {
			"action": "skip",
			"data": {
				"reason": reason
			}
		}
	});
	let res = serde_value_to_dynamic(&res);

	Ok(res)
}

// endregion: --- Rhai Functions
