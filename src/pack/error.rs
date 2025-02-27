use camino::Utf8PathBuf;
use derive_more::{Display, From};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Display)]
pub enum Error {
	#[from]
	#[display("{_0}")]
	Custom(String),

	AipackTomlMissing(Utf8PathBuf),
	VersionMissing(Utf8PathBuf),
	NamespaceMissing(Utf8PathBuf),
	NameMissing(Utf8PathBuf),

	// -- Externals
	#[from]
	Io(std::io::Error),
	#[from]
	TomlDe(toml::de::Error),

	Zip(String),
}

// region:    --- Custom

impl Error {
	pub fn custom(val: impl std::fmt::Display) -> Self {
		Self::Custom(val.to_string())
	}
}

impl From<&str> for Error {
	fn from(val: &str) -> Self {
		Self::Custom(val.to_string())
	}
}

// endregion: --- Custom

// region:    --- Error Boilerplate

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
