//! Defines the `web` module, used in the lua engine
//!
//! ---
//!
//! ## Lua documentation
//! This module exposes functions that process text.
//!
//! ### Functions
//! * `utils.web.get(url: string) -> string`

use crate::hub::get_hub;
use crate::run::RuntimeContext;
use crate::{Error, Result};
use mlua::{Lua, Table, Value};
use reqwest::redirect::Policy;
use reqwest::Client;

pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	let git_restore_fn = lua.create_function(move |lua, (url,): (String,)| web_get(lua, url))?;

	table.set("get", git_restore_fn)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Will web get
///
/// For Success, the WebResponse is
/// ```lua
/// {
///  success = true,
///  status = number,
///  url = string,
///  content = string,
/// }
/// ```
/// In case of error, the WebError is
///
/// ```lua
/// {
///  success = false,
///  status  = number | nill,
///  url     = string,
///  error   = string,
/// }
/// ```
///
fn web_get(lua: &Lua, url: String) -> mlua::Result<Value> {
	let rt = tokio::runtime::Handle::try_current().map_err(Error::TokioTryCurrent)?;
	let res: mlua::Result<Value> = tokio::task::block_in_place(|| {
		rt.block_on(async {
			let client = Client::builder()
				.redirect(Policy::limited(5)) // Set to follow up to 5 redirects
				.build()
				.map_err(crate::Error::from)?;

			let res: mlua::Result<Value> = match client.get(&url).send().await {
				Ok(response) => {
					//
					let status = response.status();
					// let url = response.url().to_string();
					let success = status.is_success();
					let status_code = response.status().as_u16() as i64;

					if success {
						// TODO: needs to reformat this error to match the lua function
						let content = response.text().await.map_err(Error::Reqwest)?;
						let res = lua.create_table()?;
						res.set("success", true)?;
						res.set("status", status_code)?;
						res.set("url", &*url)?;
						res.set("content", content)?;
						Ok(Value::Table(res))
					} else {
						let res = lua.create_table()?;
						res.set("success", false)?;
						res.set("status", status_code)?;
						res.set("url", &*url)?;
						res.set("error", format!("Not a 2xx status code ({status_code})"))?;
						// NOTE: This is not an error, as the web request was sent
						Ok(Value::Table(res))
					}
				}
				Err(err) => Err(crate::Error::Lua(format!(
					"\
Fail to do utils.web.get for url: {url}
Cause: {err}"
				))
				.into()),
			};

			if res.is_ok() {
				get_hub().publish_sync(format!("-> lua web::get OK ({}) ", url));
			}

			// return the Result<Dynamic, Error>
			res
		})
	});

	res
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, run_reflective_agent};
	use value_ext::JsonValueExt;

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_web_get_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_script = r#"
local url = "https://phet-dev.colorado.edu/html/build-an-atom/0.0.0-3/simple-text-only-test-page.html"
return utils.web.get(url)
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
	async fn test_lua_web_get_invalid_url() -> Result<()> {
		// -- Setup & Fixtures
		let fx_script = r#"
local url = "https://this-cannot-go/anywhere-or-can-it.devai"
utils.web.get(url)

		"#;

		// -- Exec
		let Err(err) = run_reflective_agent(fx_script, None).await else {
			return Err("Should have been error".into());
		};

		// TODO: need to have full object here.
		let err = err.to_string();

		// -- Check
		// TODO: Need to have better way to capture structured lua error
		assert_contains(&err, "Fail to do utils.web.get"); // success = false
		assert_contains(&err, "https://this-cannot-go/anywhere-or-can-it.devai"); // error message

		Ok(())
	}
}

// endregion: --- Tests
