// Defines the `web` module, used in the rhai engine.
//
// ---
//
// ## RHAI documentation
// The `web` module exposes functions used to perform web requests.
//
// ### Functions
// * `web::get(url: string) -> {content: null | string, status: number, url: string, success: bool, error: null | string}`

use crate::hub::get_hub;
use crate::script::DynamicMap;
use crate::Error;
use rhai::plugin::RhaiResult;
use rhai::{Dynamic, FuncRegistration, Module};

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
/// web::get(url: string) -> {content: null | string, status: number, url: string, success: bool, error: null | string}
/// ```
///
/// This function is used to perform a GET request to the specified URL.
/// It returns a map containing the content of the response, the status code, the URL, a success flag, and an error message if applicable.
///
/// For example, in a Rhai code block:
///
/// ```rhai
/// // When the request is successful
/// let result = web::get("https://example.com");
/// // result = { content: "HTML content here", status: 200, url: "https://example.com", success: true, error: null }
///
/// // When the request fails
/// let result = web::get("https://invalid-url.com");
/// // result = { content: null, status: null, url: "https://invalid-url.com", success: false, error: "Error message here" }
/// ```
fn get(url: &str) -> RhaiResult {
	let rt = tokio::runtime::Handle::try_current().map_err(Error::TokioTryCurrent)?;
	let res: Result<Dynamic, Error> = tokio::task::block_in_place(|| {
		rt.block_on(async {
			let res: Dynamic = match reqwest::get(url).await {
				Ok(response) => {
					//
					let status = response.status().as_u16() as i64;
					let content = response.text().await.map_err(Error::Reqwest)?;
					DynamicMap::default()
						.insert("success", true)
						.insert("status", status)
						.insert("url", url)
						.insert("content", content)
						.into()
				}
				Err(err) => {
					let status = err.status().map(|s| s.as_u16());
					let map = DynamicMap::default()
						.insert("success", false)
						.insert("status", status)
						.insert("url", url)
						.insert("content", ())
						.insert("error", err.to_string());
					map.into()
				}
			};
			Ok(res)
		})
	});

	let res = match res {
		Ok(res) => {
			get_hub().publish_sync(format!("-> Rhai web::get OK ({}) ", url));
			res
		}
		Err(err) => return Err(err.into()),
	};

	Ok(res)
}

// endregion: --- Rhai Functions

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, run_reflective_agent};
	use serde_json::Value;
	use value_ext::JsonValueExt;

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_rhai_web_get_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_script = r#"
let url = "https://phet-dev.colorado.edu/html/build-an-atom/0.0.0-3/simple-text-only-test-page.html";		
return web::get(url);
		"#;

		// -- Exec
		let res = run_reflective_agent(fx_script, None).await?;

		// -- Check
		let content = res.x_get_str("content")?;
		assert_contains(content, "This page tests that simple text can be");
		assert_eq!(res.x_get_i64("status")?, 200, "status code");
		assert!(res.x_get_bool("success")?, "success should be true");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_rhai_web_get_invalid_url() -> Result<()> {
		// -- Setup & Fixtures
		let fx_script = r#"
let url = "https://this-cannot-go/anywhere-or-can-it.devai";		
return web::get(url);
		"#;

		// -- Exec
		let res = run_reflective_agent(fx_script, None).await?;

		// -- Check
		assert!(
			matches!(res.get("content"), Some(Value::Null)),
			"content should be null"
		);
		assert!(matches!(res.get("status"), Some(Value::Null)), "status should be null");
		assert_eq!(res.x_get_str("url")?, "https://this-cannot-go/anywhere-or-can-it.devai");
		assert!(!res.x_get_bool("success")?, "success should be false");
		assert!(res.x_get_str("error")?.len() > 5, "should have some error message");

		Ok(())
	}
}

// endregion: --- Tests
