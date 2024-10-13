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

/// ## RHAI Documentation
/// ```rhai
/// action_skip() -> SkipActionDict
/// ```
///
/// This is to be used in the `# Data` section to return a devai skip action so that the item is not
/// included in the next flow (instruction > AI > data)
///
/// for example, in # Data rhai code block:
///
/// ```rhai
/// if item.name == "mod.rs" {
///   return devai::action_skip();
/// }
/// ```
fn action_skip() -> RhaiResult {
	let res = json!({
		"_devai_": {
			"action": "skip"
		}
	});
	let res = serde_value_to_dynamic(&res);

	Ok(res)
}

/// ## RHAI Documentation
/// ```rhai
/// action_skip(reason: string) -> SkipActionDict
/// ```
///
/// This is to be used in the `# Data` section to return a devai skip action so that the item is not
/// included in the next flow (instruction > AI > data).
///
/// This `action_skip` function takes a reason so that it get printed.
///
/// for example, in # Data rhai code block:
///
/// ```rhai
/// if item.name == "mod.rs" {
///   return devai::action_skip("mod.rs does not need to be process by this agent");
/// }
/// ```
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
