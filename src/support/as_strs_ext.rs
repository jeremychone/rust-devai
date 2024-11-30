use std::borrow::Cow;

pub trait AsStrsExt {
	fn x_as_strs(&self) -> Vec<&str>;
}

impl AsStrsExt for Vec<String> {
	fn x_as_strs(&self) -> Vec<&str> {
		self.iter().map(|s| s.as_str()).collect()
	}
}

impl AsStrsExt for Vec<Cow<'_, str>> {
	fn x_as_strs(&self) -> Vec<&str> {
		self.iter().map(|s| s.as_ref()).collect()
	}
}

impl AsStrsExt for Vec<&String> {
	fn x_as_strs(&self) -> Vec<&str> {
		self.iter().map(|s| s.as_str()).collect()
	}
}
