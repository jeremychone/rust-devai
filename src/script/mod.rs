//! Base module for the script engine.
//!
//! NOTE: At this point, LUA is the only planned scripting language for devai.
//!       It is small, simple, relatively well-known, efficient, and in many ways was made for these kinds of use cases.
//!

// region:    --- Modules

mod devai_custom;
mod lua_script;

pub use devai_custom::*;
pub use lua_script::*;

// endregion: --- Modules
