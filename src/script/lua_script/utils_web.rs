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
use crate::support::StrExt as _;
use crate::{Error, Result};
use mlua::{Lua, LuaSerdeExt, Table, Value};
use reqwest::redirect::Policy;
use reqwest::{Client, Response, header};

pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	let web_get_fn = lua.create_function(move |lua, (url,): (String,)| web_get(lua, url))?;
	let web_post_fn = lua.create_function(move |lua, (url, data): (String, Value)| web_post(lua, url, data))?;

	table.set("get", web_get_fn)?;
	table.set("post", web_post_fn)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// ```lua
/// local web_response = utils.web.get("https://google.com")
/// ```
///
/// For Success, the WebResponse is
/// ```lua
/// {
///  success = bool,
///  status = number,
///  url = string,
///  content = string,
/// }
/// ```
///
/// Note will not throw error if status is not 2xx,
/// but will throw error if the web request cannot be made.
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
				Ok(response) => get_lua_response_value(lua, response, &url).await,
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

/// ## Lua Documentation
///
/// ```lua
/// -- POST with plain text
/// local web_response = utils.web.post("https://example.com/api", "plain text data")
///
/// -- POST with JSON data
/// local web_response = utils.web.post("https://example.com/api", { key1 = "value1", key2 = "value2" })
/// ```
///
/// For Success, the WebResponse is
/// ```lua
/// {
///  success = bool,
///  status = number,
///  url = string,
///  -- If respose content-type is application/json, content will be a table (Value). Otherwise, it will be a string.
///  content = string or table,
/// }
/// ```
///
/// Note will not throw error if status is not 2xx,
/// but will throw error if the web request cannot be made.
///
fn web_post(lua: &Lua, url: String, data: Value) -> mlua::Result<Value> {
	let rt = tokio::runtime::Handle::try_current().map_err(Error::TokioTryCurrent)?;
	let res: mlua::Result<Value> = tokio::task::block_in_place(|| {
		rt.block_on(async {
			let client = Client::builder()
				.redirect(Policy::limited(5)) // Set to follow up to 5 redirects
				.build()
				.map_err(crate::Error::from)?;

			let mut request_builder = client.post(&url);

			// Set Content-Type and body based on the type of 'data'
			match data {
				Value::String(s) => {
					request_builder = request_builder
						.header(header::CONTENT_TYPE, "plain/text")
						.body(s.to_string_lossy());
				}
				Value::Table(table) => {
					let json: serde_json::Value = serde_json::to_value(table).map_err(|err| {
						crate::Error::custom(format!(
							"Cannot searlize to json the argument given to the post.\n    Cause: {err}"
						))
					})?;
					// mlua provides the serialize features.
					request_builder = request_builder
						.header(header::CONTENT_TYPE, "application/json")
						.body(json.to_string());
				}
				_ => {
					return Err(mlua::Error::RuntimeError(
						"Data must be a string or a table".to_string(),
					));
				}
			}

			let res: mlua::Result<Value> = match request_builder.send().await {
				Ok(response) => get_lua_response_value(lua, response, &url).await,
				Err(err) => Err(crate::Error::Lua(format!(
					"\
Fail to do utils.web.post for url: {url}
Cause: {err}"
				))
				.into()),
			};

			if res.is_ok() {
				get_hub().publish_sync(format!("-> lua web::post OK ({}) ", url));
			}

			// return the Result<Dynamic, Error>
			res
		})
	});

	res
}

// region:    --- Support

async fn get_lua_response_value(lua: &Lua, response: Response, url: &str) -> mlua::Result<Value> {
	let content_type = get_content_type(&response);
	//
	let status = response.status();
	let success = status.is_success();
	let status_code = status.as_u16() as i64;

	if success {
		// TODO: needs to reformat this error to match the lua function
		let res = lua.create_table()?;
		res.set("success", true)?;
		res.set("status", status_code)?;
		res.set("url", url)?;
		let content = response.text().await.map_err(Error::Reqwest)?;
		let content = get_content_value_for_content_type(lua, content_type, &content)?;
		res.set("content", content)?;
		Ok(Value::Table(res))
	} else {
		let res = lua.create_table()?;
		res.set("success", false)?;
		res.set("status", status_code)?;
		res.set("url", url)?;
		let content = response.text().await.unwrap_or_default();
		let content = Value::String(lua.create_string(&content)?);

		res.set("content", content)?;
		res.set("error", format!("Not a 2xx status code ({status_code})"))?;
		// NOTE: This is not an error, as the web request was sent
		Ok(Value::Table(res))
	}
}

/// Returns the appropriate lua Value type depending of the content type.
/// - If `application/json` it will be a Value::Table
/// - If anything else (for now), will be Value::String
fn get_content_value_for_content_type(lua: &Lua, content_type: Option<String>, content: &str) -> Result<Value> {
	let content: Value = if content_type.x_contains("application/json") {
		// parse content as json
		let content: serde_json::Value = serde_json::from_str(content)
			.map_err(|err| crate::Error::custom(format!("Fail to parse web response as json.\n    Cause: {err}")))?;

		lua.to_value(&content)?
	} else {
		Value::String(lua.create_string(content)?)
	};
	Ok(content)
}

fn get_content_type(response: &Response) -> Option<String> {
	response
		.headers()
		.get(header::CONTENT_TYPE)
		.map(|h| h.to_str().unwrap_or_default().to_lowercase())
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, setup_lua};
	use value_ext::JsonValueExt;

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_web_get_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::init_module, "web")?;
		let script = r#"
local url = "https://phet-dev.colorado.edu/html/build-an-atom/0.0.0-3/simple-text-only-test-page.html"
return utils.web.get(url)
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let content = res.x_get_str("content")?;
		assert_contains(content, "This page tests that simple text can be");
		assert_eq!(res.x_get_i64("status")?, 200, "status code");
		assert!(res.x_get_bool("success")?, "success should be true");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_web_post_json_ok() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::init_module, "web")?;
		let script = r#"
local url = "https://httpbin.org/post"
return utils.web.post(url, {some = "stuff"})
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let content = res.pointer("/content").ok_or("Should have content")?;
		assert_eq!(content.x_get_str("/json/some")?, "stuff");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_web_get_invalid_url() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::init_module, "web")?;
		let script = r#"
local url = "https://this-cannot-go/anywhere-or-can-it.aip"
return utils.web.get(url)
		"#;

		// -- Exec
		let err = match eval_lua(&lua, script) {
			Ok(_) => return Err("Should have returned an error".into()),
			Err(e) => e,
		};

		// -- Check
		let err_str = err.to_string();
		assert_contains(&err_str, "Fail to do utils.web.get");
		assert_contains(&err_str, "https://this-cannot-go/anywhere-or-can-it.aip");

		Ok(())
	}
}

// endregion: --- Tests
