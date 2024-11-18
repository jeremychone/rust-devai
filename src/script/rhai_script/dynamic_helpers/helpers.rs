//! Other dynamic helpers

use crate::{Error, Result};
use rhai::Dynamic;

/// Make a Dynamic of type String or type Array of String, as `Vec<String>`
pub fn dynamic_into_strings(dynamic: Dynamic, err_suffix: &'static str) -> Result<Vec<String>> {
	let values: Vec<String> = if let Ok(single) = dynamic.as_immutable_string_ref() {
		// `take_immutable_string` consumes the value and gives an owned `String`
		vec![(*single).to_string()]
	} else if let Ok(arr) = dynamic.as_array_ref() {
		// Collect each item from the array and try to cast it as `String`
		arr.iter()
			.filter_map(|item| item.as_immutable_string_ref().ok().map(|v| (*v).to_string()))
			.collect::<Vec<_>>()
	} else {
		// Return an error if neither conversion worked
		return Err(Error::custom(format!(
			"'{err_suffix}' is not of type String or Array Of String"
		)));
	};

	Ok(values)
}
