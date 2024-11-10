use crate::support::strings::truncate_with_ellipsis;
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

// region:    --- Support Types

pub enum DataContainer<'a> {
	Slice(&'a [&'a str]),
	Str(&'a str),
}

impl std::fmt::Debug for DataContainer<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			DataContainer::Slice(slice) => write!(f, "{:?}", slice),
			DataContainer::Str(string) => {
				let s = truncate_with_ellipsis(string, 64);
				write!(f, "{:?}", s)
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

impl<'a> DataContainer<'a> {
	fn contains(&self, val: &str) -> bool {
		match self {
			DataContainer::Slice(slice) => slice.contains(&val),
			DataContainer::Str(string) => string.contains(val),
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
}

// endregion: --- Tests
