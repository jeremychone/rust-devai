//! Module about AI Support functions.

use crate::support::cred::get_or_prompt_api_key;
use crate::Result;
use genai::resolver::AuthData;
use genai::{Client, ModelIden};

pub fn get_genai_client() -> Result<genai::Client> {
	let client = Client::builder()
		.with_auth_resolver_fn(|model: ModelIden| {
			let key_from_env = model
				.adapter_kind
				.default_key_env_name()
				.and_then(|env_name| std::env::var(env_name).ok());

			if let Some(key) = key_from_env {
				Ok(Some(AuthData::from_single(key)))
			} else {
				// TODO: needs to pass the model
				let key = get_or_prompt_api_key().map_err(|err| genai::resolver::Error::Custom(err.to_string()))?;
				Ok(Some(AuthData::from_single(key)))
			}
		})
		.build();

	Ok(client)
}
