use crate::support::text::truncate_with_ellipsis;
use std::convert::Into;
use std::fmt::Formatter;

pub fn assert_contains<'a, T>(data: T, val: &str)
where
	T: Into<DataContainer<'a>>,
{
	let container: DataContainer = data.into();
	assert!(
		container.contains(val),
		"Should contain: {}\nBut was: {:?}",
		val,
		container
	);
}

pub fn assert_not_contains<'a, T>(data: T, val: &str)
where
	T: Into<DataContainer<'a>>,
{
	let container: DataContainer = data.into();
	assert!(
		!container.contains(val),
		"Should not contain: {}\nBut was: {:?}",
		val,
		container
	);
}

pub fn assert_ends_with<'a, T>(data: T, val: &str)
where
	T: Into<DataContainer<'a>>,
{
	let container: DataContainer = data.into();
	assert!(
		container.ends_with(val),
		"Should end with: {}\nBut was: {:?}",
		val,
		container
	);
}

pub fn assert_not_ends_with<'a, T>(data: T, val: &str)
where
	T: Into<DataContainer<'a>>,
{
	let container: DataContainer = data.into();
	assert!(
		!container.ends_with(val),
		"Should not end with: {}\nBut was: {:?}",
		val,
		container
	);
}

// region:    --- Support Types

pub enum DataContainer<'a> {
	Slice(&'a [&'a str]),
	Str(&'a str),
}

impl std::fmt::Debug for DataContainer<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			DataContainer::Slice(slice) => write!(f, "{:?}", slice),
			DataContainer::Str(s) => {
				let s = truncate_with_ellipsis(s, 64, "...");
				write!(f, "{s}")
			}
		}
	}
}

impl<'a> From<&'a [&'a str]> for DataContainer<'a> {
	fn from(slice: &'a [&'a str]) -> Self {
		DataContainer::Slice(slice)
	}
}

impl<'a> From<&'a Vec<&'a str>> for DataContainer<'a> {
	fn from(vec: &'a Vec<&'a str>) -> Self {
		DataContainer::Slice(&vec[..])
	}
}

impl<'a> From<&'a str> for DataContainer<'a> {
	fn from(string: &'a str) -> Self {
		DataContainer::Str(string)
	}
}

impl<'a> From<&'a String> for DataContainer<'a> {
	fn from(string: &'a String) -> Self {
		DataContainer::Str(string)
	}
}

impl DataContainer<'_> {
	fn contains(&self, val: &str) -> bool {
		match self {
			DataContainer::Slice(slice) => slice.contains(&val),
			DataContainer::Str(string) => string.contains(val),
		}
	}

	fn ends_with(&self, val: &str) -> bool {
		match self {
			DataContainer::Slice(slice) => {
				if slice.is_empty() {
					return false;
				}
				slice.last().unwrap().ends_with(val)
			},
			DataContainer::Str(string) => string.ends_with(val),
		}
	}
}

// endregion: --- Support Types

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_assert_contains() -> Result<()> {
		let data_vec = vec!["apple", "banana", "cherry"];
		assert_contains(&data_vec, "banana");

		let data_slice: &[&str] = &["dog", "cat", "mouse"];
		assert_contains(data_slice, "cat");

		let data_str = "This is a test string";
		assert_contains(data_str, "test");

		Ok(())
	}

	#[test]
	fn test_test_support_asserts_ends_with_simple() -> Result<()> {
		// -- Setup & Fixtures
		let data_vec = vec!["apple", "banana", "cherry"];
		let data_slice: &[&str] = &["dog", "cat", "mouse"];
		let data_str = "This is a test string";
		
		// -- Exec & Check
		assert_ends_with(&data_vec, "y");
		assert_ends_with(data_slice, "e");
		assert_ends_with(data_str, "string");
		
		assert_not_ends_with(&data_vec, "x");
		assert_not_ends_with(data_slice, "dog");
		assert_not_ends_with(data_str, "test");

		Ok(())
	}
}

// endregion: --- Tests
