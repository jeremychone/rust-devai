use rhai::plugin::RhaiResult;
use rhai::{FuncRegistration, Module};

pub fn rhai_module() -> Module {
	// Create a module for text functions
	let mut module = Module::new();

	FuncRegistration::new("restore")
		.in_global_namespace()
		.set_into_module(&mut module, git_restore);

	module
}

// region:    --- Functions

fn git_restore(path: &str) -> RhaiResult {
	let output = std::process::Command::new("git")
		.arg("restore")
		.arg(path)
		.output()
		.expect("Failed to execute command");

	let stdout = String::from_utf8_lossy(&output.stdout);
	let stderr = String::from_utf8_lossy(&output.stderr);

	if !stderr.is_empty() {
		println!("stderr: {}", stderr);
		return Err(stderr.into());
	}

	Ok(stdout.to_string().into())
}

// endregion: --- Functions
