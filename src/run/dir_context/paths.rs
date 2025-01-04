//! All Paths for devai

// -- Devai Base
// Will be from the home dir
pub const DEVAI_BASE: &str = ".devai-base";

// -- Relative to ~/.devai-base (and later to .devai/)
pub const CUSTOM_AGENT: &str = "custom/agent";
pub const CUSTOM_LUA: &str = "custom/lua";

// -- For the .devai/ (later, will be just "agent" rather than "command-agent" )
pub const DEVAI_AGENT_DEFAULT_DIR: &str = "default/command-agent";
pub const DEVAI_AGENT_CUSTOM_DIR: &str = "custom/command-agent";
