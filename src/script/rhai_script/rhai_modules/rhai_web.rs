//! Defines the `web` module, used in the rhai engine.
//!
//! ---
//!
//! ## RHAI documentation
//! The `web` module exposes functions used to perform web requests.
//!
//! ### Functions
//! * `web::get(url: string) -> String`

use crate::hub::get_hub;
use crate::Error;
use rhai::plugin::RhaiResult;
use rhai::{FuncRegistration, Module};

pub fn rhai_module() -> Module {
	// Create a module for web functions
	let mut module = Module::new();

	FuncRegistration::new("get")
		.in_global_namespace()
		.set_into_module(&mut module, get);

	module
}

// region:    --- Rhai Functions

/// ## RHAI Documentation
/// ```rhai
/// web::get(url: string) -> String
/// ```
///
/// Fetches the content of the specified URL and returns it as a string.
fn get(url: &str) -> RhaiResult {
	let rt = tokio::runtime::Handle::try_current().map_err(Error::TokioTryCurrent)?;
	let res = tokio::task::block_in_place(|| {
		rt.block_on(async {
			let response = match reqwest::get(url).await {
				Ok(response) => response,
				Err(err) => return Err(Error::Reqwest(err)),
			};
			response.text().await.map_err(Error::Reqwest)
		})
	});

	let text = match res {
		Ok(text) => {
			get_hub().publish_sync(format!("-> Rhai web::get OK ({}) ", url));
			text
		}
		Err(err) => return Err(err.into()),
	};

	Ok(text.into())
}

// endregion: --- Rhai Functions
