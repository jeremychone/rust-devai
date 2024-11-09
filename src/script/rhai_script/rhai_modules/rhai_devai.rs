//! Defines the `devai` module, used in the rhai engine
//!
//! ---
//!
//! ## RHAI documentation
//! The `devai` module exposes functions that modify the default flow of the
//! devai parser and script runner.
//!
//! ### Functions
//! * `devai::action_skip() -> SkipActionDict`
//! * `devai::action_skip(reason: string) -> SkipActionDict`

use crate::agent::{find_agent, LocatorMode};
use crate::run::RunBaseOptions;
use crate::run::{run_command_agent, RuntimeContext};
use crate::script::rhai_script::dynamic_helpers::{dynamics_to_values, value_to_dynamic};
use crate::Error;
use rhai::plugin::RhaiResult;
use rhai::{Dynamic, FuncRegistration, Module};
use serde_json::{json, Value};

pub fn rhai_module(runtime_context: &RuntimeContext) -> Module {
	// Create a module for text functions
	let mut module = Module::new();

	FuncRegistration::new("action_skip")
		.in_global_namespace()
		.set_into_module(&mut module, action_skip);

	FuncRegistration::new("action_skip")
		.in_global_namespace()
		.set_into_module(&mut module, action_skip_with_reason);

	let ctx = runtime_context.clone();
	FuncRegistration::new("run")
		.in_global_namespace()
		.set_into_module(&mut module, move |cmd_agent: &str, items: Vec<Dynamic>| {
			run_with_items(&ctx, cmd_agent, items)
		});

	module
}

// region:    --- run...

fn run_with_items(ctx: &RuntimeContext, cmd_agent: &str, items: Vec<Dynamic>) -> RhaiResult {
	let items = dynamics_to_values(items)?;
	// TODO: Might want to reuse the current one
	let agent = find_agent(cmd_agent, ctx.dir_context(), LocatorMode::DevaiParentDir)?;

	let rt = tokio::runtime::Handle::try_current().map_err(Error::TokioTryCurrent)?;

	// Note: Require to have
	let runtime = ctx.get_runtime()?;
	let res = tokio::task::block_in_place(|| {
		rt.block_on(async { run_command_agent(&runtime, &agent, Some(items), &RunBaseOptions::default(), true).await })
	});

	let res = res?;

	let rhai_val = if let Some(values) = res {
		value_to_dynamic(&Value::Array(values))
	} else {
		Dynamic::UNIT
	};

	Ok(rhai_val)
}

// endregion: --- run...

// region:    --- action_skip..

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
	// TODO: need to create the Dynamic directly,
	//       no need to passthrough json -> Dynamic -> json later
	let res = json!({
		"_devai_": {
			"kind": "ActionSkip"
		}
	});
	let res = value_to_dynamic(&res);

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
			"kind": "ActionSkip",
			"data": {
				"reason": reason
			}
		}
	});
	let res = value_to_dynamic(&res);

	Ok(res)
}

// endregion: --- action_skip..

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Error = Box<dyn std::error::Error>;
	type Result<T> = core::result::Result<T, Error>; // For tests.

	use crate::_test_support::run_reflective_agent;
	use serde_json::from_value;

	// Note: multi_thread required, because rhai devai::run is a sync calling a async.
	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_rhai_devai_run_simple() -> Result<()> {
		let res = run_reflective_agent(r#"return devai::run("./agent-hello.md", ["one", "two"])"#, None).await;

		// NOTE: apparently when multi thread, need to print error
		let res = match res {
			Ok(res) => res,
			Err(err) => {
				panic!("test_rhai_devai_run_simple ERROR: {err}");
			}
		};

		let vals: Vec<String> = from_value(res)?;

		assert_eq!(
			vals,
			["hello 'one' from agent-hello.md", "hello 'two' from agent-hello.md"]
		);
		Ok(())
	}
}

// endregion: --- Tests
