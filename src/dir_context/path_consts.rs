//! All Paths for Aipack

// -- Aipack Base

// Will be from the home dir
pub const AIPACK_BASE: &str = ".aipack-base";

// -- .aipack/

pub const AIPACK_DIR_NAME: &str = ".aipack";

pub const CONFIG_FILE_NAME: &str = "config.toml";

// -- Common Path (for .aipack/ and ~/.aipack-base/)

// TODO: probably need to add a common lua, or perhaps allow `require("jc@utils/lua/somefile")`
// pub const CUSTOM_LUA_DIR: &str = "custom/lua";

pub const PACK_CUSTOM: &str = "pack/custom";
pub const PACK_INSTALLED: &str = "pack/installed";
pub const PACK_DOWNLOAD: &str = "pack/.download";

// -- New Agent Templates
