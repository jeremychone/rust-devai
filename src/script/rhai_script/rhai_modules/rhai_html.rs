//! Defines the `html` module, used in the Rhai engine.
//!
//! ---
//!
//! ## RHAI documentation
//! The `html` module exposes functions used to process HTML content.
//!
//! ### Functions
//! * `html::prune_to_content(html_content: string) -> string`

use crate::support::html::prune_to_content;
use rhai::plugin::RhaiResult;
use rhai::{Dynamic, EvalAltResult, FuncRegistration, Module};

pub fn rhai_module() -> Module {
	// Create a module for HTML functions
	let mut module = Module::new();

	FuncRegistration::new("prune_to_content")
		.in_global_namespace()
		.set_into_module(&mut module, prune_to_content_rhai);

	module
}

// region:    --- Rhai Functions

/// ## RHAI Documentation
/// ```rhai
/// html::prune_to_content(html_content: string) -> string
/// ```
///
/// Strips non-content elements from the provided HTML content and returns the cleaned HTML as a string.
///
/// This function removes:
/// - Non-visible tags such as `<script>`, `<link>`, `<style>`, and `<svg>`.
/// - HTML comments.
/// - Empty lines.
/// - Attributes except for `class`, `aria-label`, and `href`.
///
/// # Arguments
///
/// * `html_content` - A `String` containing the HTML content to be processed.
///
/// # Returns
///
/// A `String` containing the cleaned HTML content.
fn prune_to_content_rhai(html_content: &str) -> RhaiResult {
	match prune_to_content(html_content.to_string()) {
		Ok(cleaned_html) => Ok(Dynamic::from(cleaned_html)),
		Err(err) => Err(Box::new(EvalAltResult::ErrorRuntime(
			format!("Failed to prune HTML content: {}", err).into(),
			rhai::Position::NONE,
		))),
	}
}

// endregion: --- Rhai Functions
