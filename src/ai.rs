//! Module about AI Support functions.

use crate::support::cred::get_or_prompt_api_key;
use crate::Result;
use genai::chat::{ChatMessage, ChatRequest};
use genai::resolver::AuthData;
use genai::{Client, ModelIden};
use simple_fs::SFile;
use std::fs;

const MODEL: &str = "gpt-4o-mini";

pub fn get_genai_client() -> Result<genai::Client> {
	let client = Client::builder()
		.with_auth_resolver_fn(|model: ModelIden| {
			let key = get_or_prompt_api_key().map_err(|err| genai::resolver::Error::Custom(err.to_string()))?;
			Ok(Some(AuthData::from_single(key)))
		})
		.build();

	Ok(client)
}

pub async fn run_ai_on_file(client: &Client, inst: &str, file: SFile) -> Result<()> {
	// Read from SFile
	let file_content = fs::read_to_string(&file)?;

	let chat_req = ChatRequest::from_system(inst).append_message(ChatMessage::user(file_content));

	let res = client.exec_chat(MODEL, chat_req, None).await?;

	if let Some(new_content) = res.content_text_into_string() {
		let new_content = clean_rust_content(new_content)?;
		fs::write(&file, new_content)?;
	}

	Ok(())
}

fn clean_rust_content(content: String) -> Result<String> {
	let trimmed_content = content.trim();

	// Check if the content starts with ```rust and ends with ```
	if trimmed_content.starts_with("```rust") && trimmed_content.ends_with("```") {
		// Remove the first ```rust and the last ```
		let without_rust_tag = &trimmed_content["```rust".len()..];
		let cleaned_content = &without_rust_tag[..without_rust_tag.len() - 3]; // Remove the ending ```

		// Return the cleaned content, trimmed of leading/trailing whitespace
		Ok(cleaned_content.trim().to_string())
	} else {
		Ok(trimmed_content.to_string())
	}
}
