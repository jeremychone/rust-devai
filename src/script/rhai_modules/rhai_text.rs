use crate::support::decode_html_entities;
use rhai::plugin::RhaiResult;
use rhai::{FuncRegistration, Module};

pub fn rhai_module() -> Module {
	// Create a module for text functions
	let mut module = Module::new();

	// Register the functions to the module

	FuncRegistration::new("escape_decode")
		.in_global_namespace()
		.set_into_module(&mut module, escape_decode);

	FuncRegistration::new("escape_decode_if_needed")
		.in_global_namespace()
		.set_into_module(&mut module, escape_decode_if_needed);

	module
}

// region:    --- Rhai Functions

/// Only escape if needed. right now, the test only test `&lt;`
fn escape_decode_if_needed(content: &str) -> RhaiResult {
	if !content.contains("&lt;") {
		Ok(content.into())
	} else {
		escape_decode(content)
	}
}

// html-escape
fn escape_decode(content: &str) -> RhaiResult {
	let decoded = decode_html_entities(content);
	Ok(decoded.into())
}

// endregion: --- Rhai Functions
