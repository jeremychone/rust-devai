use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
	// -- Cli Command
	AgentNotFound(String),

	// -- Run
	CannotRunMissingModel,

	// -- Sub Modules
	#[from]
	DynamicSupport(crate::script::DynamicSupportError),

	// -- Externals
	#[from]
	Toml(toml::de::Error),
	#[from]
	JsonValueExt(value_ext::JsonValueExtError),
	#[from]
	Serde(serde_json::Error),
	#[from]
	Rhai(rhai::EvalAltResult),
	#[from]
	Handlebars(handlebars::RenderError),
	#[from]
	GenAI(genai::Error),
	#[from]
	SimpleFs(simple_fs::Error),
	#[from]
	Keyring(keyring::Error),
	#[from]
	Clap(clap::error::Error),
	#[from]
	Io(std::io::Error),

	// -- Custom
	#[from]
	Custom(String),
}

// region:    --- Froms

impl From<Box<rhai::EvalAltResult>> for Error {
	fn from(val: Box<rhai::EvalAltResult>) -> Self {
		Self::Rhai(*val)
	}
}

// endregion: --- Froms

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

impl core::fmt::Display for Error {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
