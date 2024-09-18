use crate::Result;
use keyring::Entry;
use std::env;
use std::io::{self, Write};

const KEY_SERVICE: &str = "jc_secrets";
const KEY_NAME: &str = "OPENAI_API_KEY";

// NOT USED NOW
fn _clear_api_key() -> Result<()> {
	_clear_key(KEY_SERVICE, KEY_NAME)?;
	Ok(())
}

pub fn get_or_prompt_api_key() -> Result<String> {
	let api_key = env::var(KEY_NAME);

	if let Ok(api_key) = api_key {
		return Ok(api_key);
	}

	let api_key = get_and_save_pwd(KEY_SERVICE, KEY_NAME)?;

	Ok(api_key)
}

// region:    --- Support

// Get the value from local keychain, or prompt use to save and return.
fn get_and_save_pwd(service: &str, name: &str) -> Result<String> {
	let entry = Entry::new(service, name)?;
	let pwd = match entry.get_password() {
		Ok(pwd) => pwd,
		Err(keyring::Error::NoEntry) => prompt_and_save(entry, name)?,
		Err(other) => return Err(format!("Fail to exec keyring: {other}").into()),
	};

	Ok(pwd)
}

/// Clear the secret key if present
/// - Returns true if it was present
/// - If not present, will do nothing (and no error)
///
/// NOT USED FOR NOW
fn _clear_key(service: &str, name: &str) -> Result<bool> {
	if let Ok(entry) = Entry::new(service, name) {
		entry.delete_credential()?;
		Ok(true)
	} else {
		Ok(false)
	}
}

// Ask the user for a value and store it in the entry.
fn prompt_and_save(entry: Entry, disp_name: &str) -> Result<String> {
	// -- Prompt the user
	let mut input = String::new();
	println!(
		r#"'{disp_name}' not found in keychain. 
Please enter value: "#
	);
	io::stdout().flush()?;
	io::stdin().read_line(&mut input)?;

	// -- Validate the answer
	let val = input.trim().to_string();
	if val.is_empty() {
		return Err("Value cannot be empty.".into());
	}

	// -- Save and get again
	entry.set_password(&val)?;
	// Making sure we get the value from store
	let val = entry.get_password()?;

	Ok(val)
}

// endregion: --- Support
