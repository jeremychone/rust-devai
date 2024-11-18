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
use crate::script::DynaMap;
use crate::Error;
use reqwest::redirect::Policy;
use reqwest::Client;
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
///
/// Perform a http GET request to the specified URL and returns an response object contain `.content` for it's text content.
///
/// ```
/// // API Signature
/// web::get(url: string) -> WebGetResponse (throws: WebGetException)
/// ```
///
/// By default, it will follows up to 5 redirects.
///
/// > Note: For now, only support text based content type.
///
///
/// ### Example
/// ```
/// let response = web::get("https://britesnow.com/test/text-page.txt")
/// let content = reponse.content;
/// ```
///
/// ### Returns (WebGetResponse)
///
/// Returns when the http response status code is 2xx range (will follow up to 5 redirects).
///
/// ```
/// {
///   success: true,    // true when the "final" http request is successful (2xx range)
///   status:  number,  // The status code returned by the http request
///   url:     string,  // The full URL requested
///   content: string,  // The text content
/// }
/// ```
///
/// ### Exception (WebGetException)
///
/// ```
/// {
///   success: false,   // false when the HTTP request is not successful
///   status?: number,  // (optional) The status code returned by the HTTP request
///   url:     string,  // The full URL requested
///   error:   string,  // The error message
/// }
/// ```
fn get(url: &str) -> RhaiResult {
	let rt = tokio::runtime::Handle::try_current().map_err(Error::TokioTryCurrent)?;
	let res: Result<Dynamic, Error> = tokio::task::block_in_place(|| {
		rt.block_on(async {
			let client = Client::builder()
				.redirect(Policy::limited(5)) // Set to follow up to 5 redirects
				.build()?;

			let res: Result<Dynamic, Error> = match client.get(url).send().await {
				Ok(response) => {
					//
					let status = response.status();
					let success = status.is_success();
					let status_code = response.status().as_u16() as i64;

					if success {
						// TODO: needs to reformat this error to match the rhai function
						let content = response.text().await.map_err(Error::Reqwest)?;
						let res: Dynamic = DynaMap::default()
							.insert("success", true)
							.insert("status", status_code)
							.insert("url", url)
							.insert("content", content)
							.into();
						Ok(res)
					} else {
						let res: Dynamic = DynaMap::default()
							.insert("success", false)
							.insert("status", status_code)
							.insert("url", url)
							.insert("error", format!("Not a 2xx status code ({status_code})"))
							.into();
						Err(Error::RhaiDynamic(res))
					}
				}
				Err(err) => {
					let status = err.status().map(|s| s.as_u16());
					let map = DynaMap::default()
						.insert("success", false)
						.insert("status", status)
						.insert("url", url)
						.insert("error", err.to_string());
					let res: Dynamic = map.into();
					Err(Error::RhaiDynamic(res))
				}
			};

			// return the Result<Dynamic, Error>
			res
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
try{
 web::get(url);
 return "nothing"; // should not happen
} catch (ex) {
 return ex
}
		"#;

		// -- Exec
		let res = run_reflective_agent(fx_script, None).await?;

		// -- Check
		assert!(res.get("content").is_none(), "content should not be a property");
		assert!(matches!(res.get("status"), Some(Value::Null)), "status should be null");
		assert_eq!(res.x_get_str("url")?, "https://this-cannot-go/anywhere-or-can-it.devai");
		assert!(!res.x_get_bool("success")?, "success should be false");
		assert!(res.x_get_str("error")?.len() > 5, "should have some error message");

		Ok(())
	}
}

// endregion: --- Tests
