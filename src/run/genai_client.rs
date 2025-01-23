//! Module about AI support functions.

use crate::support::cred::get_or_prompt_api_key;
use crate::Result;
use genai::resolver::AuthData;
use genai::{Client, ModelIden};

pub fn get_genai_client() -> Result<genai::Client> {
	let client = Client::builder()
		.with_auth_resolver_fn(|model: ModelIden| {
			// -- Get the key_name, if none, then, could be ollama, so return None
			let Some(key_name) = model.adapter_kind.default_key_env_name() else {
				return Ok(None);
			};

			// -- Try to get it from the env variable
			let key_from_env = std::env::var(key_name).ok();

			if let Some(key) = key_from_env {
				Ok(Some(AuthData::from_single(key)))
			}
			// -- Otherwise, get it with keyring
			else {
				#[cfg(target_os = "macos")]
				{
					// TODO: need to pass the model
					let key = get_or_prompt_api_key(key_name)
						.map_err(|err| genai::resolver::Error::Custom(err.to_string()))?;
					Ok(Some(AuthData::from_single(key)))
				}
				#[cfg(not(target_os = "macos"))]
				{
					let msg = format!("Environment variable '{key_name}' make sure to set it for this terminal.");
					Err(genai::resolver::Error::Custom(msg.to_string()))
				}
			}
		})
		.build();

	Ok(client)
}
