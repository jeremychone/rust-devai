use crate::Result;
use crossterm::cursor::MoveTo;
use crossterm::terminal::ClearType;
use crossterm::{execute, terminal};
use keyring::Entry;
use std::io::{self, Write};

const KEY_SERVICE: &str = "aipack_secrets";

// NOT USED NOW
fn _clear_api_key(key_name: &str) -> Result<()> {
	_clear_key(KEY_SERVICE, key_name)?;
	Ok(())
}

pub fn get_or_prompt_api_key(key_name: &str) -> Result<String> {
	let api_key = get_and_save_pwd(KEY_SERVICE, key_name)?;

	Ok(api_key)
}

// region:    --- Support

// Get the value from the local keychain, or prompt the user to save and return.
fn get_and_save_pwd(service: &str, name: &str) -> Result<String> {
	let entry = Entry::new(service, name)?;
	let pwd = match entry.get_password() {
		Ok(pwd) => pwd,
		Err(keyring::Error::NoEntry) => prompt_and_save(entry, name)?,
		Err(other) => return Err(format!("Failed to execute keyring: {other}").into()),
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
	// -- exit of raw mode for now to get the input.
	terminal::disable_raw_mode().expect("Failed to disable crossterm raw mode");

	// -- Prompt the user
	let mut input = String::new();
	println!(
		r#"
API KEY for '{}' not found in environment vairable or keychain 
Please enter value (will store key in Mac keychain, under aipack_secrets/{}): "#,
		disp_name, disp_name
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

	// -- put back in raw mode
	clear_terminal()?;
	// sleep 200ms
	std::thread::sleep(std::time::Duration::from_millis(200));
	terminal::enable_raw_mode().expect("Failed to enable crossterm raw mode");

	// TODO: We need to have the TUI taking care of the prompt

	Ok(val)
}

fn clear_terminal() -> Result<()> {
	let mut stdout = io::stdout();
	execute!(stdout, terminal::Clear(ClearType::All))?;
	execute!(stdout, MoveTo(0, 0))?; // Move the cursor to the top-left
	Ok(())
}
// endregion: --- Support
