//! All Paths for devai

// -- Devai Base

// Will be from the home dir
pub const DEVAI_BASE: &str = ".devai-base";

// -- .devai/

pub const DEVAI_DIR_NAME: &str = ".devai";
pub const DEVAI_DIR_PATH: &str = "./.devai";

pub const DEVAI_CONFIG_FILE_PATH: &str = "config.toml";

// -- Common Path (for .devai/ and ~/.devai-base/)

pub const CUSTOM_AGENT_DIR: &str = "custom/agent";
pub const CUSTOM_LUA_DIR: &str = "custom/lua";

pub const DEFAULT_AGENT_DIR: &str = "default/agent";

// -- Doc
pub const DEVAI_DOC_DIR: &str = "doc";

// -- New Agent Templates

pub const DEVAI_NEW_CUSTOM_COMMAND_DIR: &str = "custom/new-template/agent";
pub const DEVAI_NEW_DEFAULT_COMMAND_DIR: &str = "default/new-template/agent";
pub const DEVAI_NEW_COMMAND_DIRS: &[&str] = &[
	// by priority
	DEVAI_NEW_CUSTOM_COMMAND_DIR,
	DEVAI_NEW_DEFAULT_COMMAND_DIR,
];

// -- New Solo Templates
pub const DEVAI_NEW_CUSTOM_SOLO_DIR: &str = "custom/new-template/solo-agent";
pub const DEVAI_NEW_DEFAULT_SOLO_DIR: &str = "default/new-template/solo-agent";
pub const DEVAI_NEW_SOLO_DIRS: &[&str] = &[
	// by priority
	DEVAI_NEW_CUSTOM_SOLO_DIR,
	DEVAI_NEW_DEFAULT_SOLO_DIR,
];
