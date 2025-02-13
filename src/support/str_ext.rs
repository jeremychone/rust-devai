//! Convenience functions for common string or container of string

pub trait StrExt {
	fn x_contains(&self, text: &str) -> bool;
}

// region:    --- Implementations on Ts

impl StrExt for String {
	fn x_contains(&self, text: &str) -> bool {
		self.contains(text)
	}
}

impl StrExt for &str {
	fn x_contains(&self, text: &str) -> bool {
		self.contains(text)
	}
}

impl StrExt for &String {
	fn x_contains(&self, text: &str) -> bool {
		self.contains(text)
	}
}

// endregion: --- Implementations on Ts

// region:    --- Implementation on Containers

impl<T: StrExt> StrExt for Option<T> {
	fn x_contains(&self, text: &str) -> bool {
		match self {
			Some(s) => s.x_contains(text),
			None => false,
		}
	}
}

impl<T: StrExt> StrExt for Vec<T> {
	fn x_contains(&self, text: &str) -> bool {
		self.iter().any(|s| s.x_contains(text))
	}
}

// endregion: --- Implementation on Containers
