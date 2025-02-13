//! Lua Management implementaitons for the crate::Error

use crate::Error;
use lazy_regex::regex;
use std::borrow::Cow;
use std::sync::Arc;

impl Error {
	pub fn from_error_with_script(lua_error: &mlua::Error, script: &str) -> Error {
		let mut buff: Vec<String> = Vec::new();
		for item in lua_error.chain() {
			if let Some(lua_item) = item.downcast_ref::<mlua::Error>() {
				let msg = lua_item.to_string();
				let msg = if msg.contains("traceback") | msg.contains("syntax") {
					process_stack_with_script(&msg, script)
				} else {
					msg
				};
				buff.push(format!("Lua error:\n{}", msg));
			} else {
				buff.push(format!("Other lua error:\n{}", item));
			}
		}
		Error::Lua(buff.join("\n"))
	}
}

fn process_stack_with_script(stack: &str, script: &str) -> String {
	let script_lines: Vec<&str> = script.lines().collect();
	let mut buff: Vec<Cow<str>> = Vec::new();

	let rx = regex!(r#"\[string .*?\]:([\d]+)(:|>)"#);

	for line in stack.lines() {
		if rx.is_match(line) {
			// Replace all occurrences of the pattern with the extracted number
			let replaced_line = rx.replace_all(line, |caps: &regex::Captures| {
				if let Some(num) = caps.get(1).and_then(|m| m.as_str().parse::<usize>().ok()) {
					if let Some(script_line) = script_lines.get(num - 1) {
						Cow::from(format!("At line {num} '{}'", script_line.trim()))
					} else {
						Cow::from(format!("Line({num})"))
					}
				} else {
					Cow::from("")
				}
			});
			buff.push(replaced_line);
		} else {
			// Add the original line if no match is found
			buff.push(line.into());
		}
	}

	buff.join("\n")
}

// region:    --- Froms

// For now, we serialize as string for sync/send
impl From<mlua::Error> for Error {
	fn from(lua_error: mlua::Error) -> Self {
		Error::from(&lua_error)
	}
}

/// Do the From mlua error without script
impl From<&mlua::Error> for Error {
	fn from(lua_error: &mlua::Error) -> Self {
		let mut buff: Vec<String> = Vec::new();
		for item in lua_error.chain() {
			if let Some(lua_item) = item.downcast_ref::<mlua::Error>() {
				buff.push(format!("Lua error chain item\n - {}", lua_item))
			} else {
				buff.push(format!("Other error chain item\n - {}", item))
			}
		}
		let msg = buff.join("\n");
		// Note: here is Self::lua, it gets a stackoverflow
		Self::Lua(msg)
	}
}

impl From<Error> for mlua::Error {
	fn from(value: Error) -> Self {
		mlua::Error::ExternalError(Arc::new(value))
	}
}

// endregion: --- Froms
