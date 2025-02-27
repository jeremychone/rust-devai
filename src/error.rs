use crate::pack;
use derive_more::From;
use derive_more::derive::Display;
use tokio::runtime::TryCurrentError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Display)]
pub enum Error {
	// -- Cli Command
	#[display("Command Agent not found at: {_0}")]
	CommandAgentNotFound(String),

	// -- Agent
	#[display("Model is missing for agent path: {agent_path}")]
	ModelMissing {
		agent_path: String,
	},

	// -- Config
	#[display("Config invalid (config path: {path})\n  reason: {reason}")]
	Config {
		path: String,
		reason: String,
	},

	// -- Pack
	#[from]
	Pack(pack::Error),

	// -- Run
	BeforeAllFailWrongReturn {
		cause: String,
	},

	// -- TokioSync
	TokioTryCurrent(TryCurrentError),

	// -- Externals / custom
	#[display("LUA ERROR - {_0}")]
	Lua(String),

	// -- Externals auto froms
	#[from]
	Serde(serde_json::Error),
	#[from]
	Toml(toml::de::Error),
	#[from]
	JsonValueExt(value_ext::JsonValueExtError),
	#[from]
	Handlebars(handlebars::RenderError),
	#[from]
	#[display("{}", _0.c_display())]
	GenAI(genai::Error),
	#[from]
	SimpleFs(simple_fs::Error),
	#[from]
	Keyring(keyring::Error),
	#[from]
	Clap(clap::error::Error),
	#[from]
	Reqwest(reqwest::Error),
	#[from]
	Io(std::io::Error),

	// -- Custom
	#[display("{_0}")]
	#[from]
	Custom(String),

	#[display("Error: {_0}\n\tCause: {_1}")]
	CustomAndCause(String, String),
}

// region:    --- Custom display

trait CustomDisplay {
	fn c_display(&self) -> String;
}

/// NOTE: Very early pass to cleanup error message (experimental)
impl CustomDisplay for genai::Error {
	fn c_display(&self) -> String {
		match self {
			genai::Error::ChatReqHasNoMessages { model_iden } => {
				format!("Chat request for model '{:?}' has no messages.", model_iden)
			}

			genai::Error::LastChatMessageIsNotUser {
				model_iden,
				actual_role,
			} => format!(
				"Last chat message in model '{:?}' is not from a user but from '{}'.",
				model_iden, actual_role
			),

			genai::Error::MessageRoleNotSupported { model_iden, role } => format!(
				"Role '{}' is not supported for messages in model '{:?}'.",
				role, model_iden
			),

			genai::Error::MessageContentTypeNotSupported { model_iden, cause } => format!(
				"Message content type '{}' is not supported in model '{:?}'.",
				cause, model_iden
			),

			genai::Error::JsonModeWithoutInstruction => {
				"JSON mode is enabled but no instruction was provided.".to_string()
			}

			genai::Error::NoChatResponse { model_iden } => {
				format!("No response received from model '{:?}'.", model_iden)
			}

			genai::Error::InvalidJsonResponseElement { info } => {
				format!("Invalid JSON response element: '{}'.", info)
			}

			genai::Error::RequiresApiKey { model_iden } => {
				format!("API key is required for model '{:?}'.", model_iden)
			}

			genai::Error::NoAuthResolver { model_iden } => {
				format!("No authentication resolver available for model '{:?}'.", model_iden)
			}

			genai::Error::NoAuthData { model_iden } => {
				format!("No authentication data found for model '{:?}'.", model_iden)
			}

			genai::Error::ModelMapperFailed { model_iden, cause } => {
				format!("Model mapping failed for '{:?}': {}.", model_iden, cause)
			}

			genai::Error::WebAdapterCall {
				adapter_kind,
				webc_error,
			} => format!("Web adapter call '{}' failed: {}.", adapter_kind, webc_error),

			genai::Error::WebModelCall { model_iden, webc_error } => {
				format!("Web model call for '{:?}' failed: {}.", model_iden, webc_error)
			}

			genai::Error::StreamParse {
				model_iden,
				serde_error,
			} => format!("Failed to parse stream for '{:?}': {}.", model_iden, serde_error),

			genai::Error::StreamEventError { model_iden, body } => {
				format!("Stream event error in model '{:?}': '{}'.", model_iden, body)
			}

			genai::Error::WebStream { model_iden, cause } => {
				format!("Web stream error in model '{:?}': {}.", model_iden, cause)
			}

			genai::Error::Resolver {
				model_iden,
				resolver_error,
			} => format!("Resolver error in model '{:?}': {}.", model_iden, resolver_error),

			genai::Error::EventSourceClone(cannot_clone_request_error) => {
				format!("Failed to clone event source request: {}.", cannot_clone_request_error)
			}

			genai::Error::JsonValueExt(json_value_ext_error) => {
				format!("JSON value extension error: {}.", json_value_ext_error)
			}

			genai::Error::ReqwestEventSource(error) => format!("Reqwest event source error: {}.", error),

			genai::Error::SerdeJson(error) => format!("Serde JSON processing error: {}.", error),

			#[allow(unreachable_patterns)] // in case genai::Error changes
			other => format!("{other:?}"),
		}
	}
}

// endregion: --- Custom display

// region:    --- Custom

impl Error {
	pub fn custom(val: impl std::fmt::Display) -> Self {
		Self::Custom(val.to_string())
	}

	pub fn custom_and_cause(context: impl Into<String>, cause: impl std::fmt::Display) -> Self {
		Self::CustomAndCause(context.into(), cause.to_string())
	}

	/// Same as custom_and_cause (just a "cute" shorcut)
	pub fn cc(context: impl Into<String>, cause: impl std::fmt::Display) -> Self {
		Self::CustomAndCause(context.into(), cause.to_string())
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
