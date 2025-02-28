use camino::Utf8PathBuf;
use derive_more::{Display, From};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Display)]
pub enum Error {
    #[from]
    #[display("{_0}")]
    Custom(String),

    #[display("pack.toml file is missing at '{_0}'")]
    AipackTomlMissing(Utf8PathBuf),
    
    #[display("version field is missing or empty in '{_0}'")]
    VersionMissing(Utf8PathBuf),
    
    #[display("namespace field is missing or empty in '{_0}'")]
    NamespaceMissing(Utf8PathBuf),
    
    #[display("name field is missing or empty in '{_0}'")]
    NameMissing(Utf8PathBuf),

    // -- Externals
    #[from]
    #[display("IO error: {_0}")]
    Io(std::io::Error),
    
    #[from]
    #[display("TOML parsing error: {_0}")]
    TomlDe(toml::de::Error),

    #[display("Zip error: {_0}")]
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
