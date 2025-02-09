// region:    --- Modules

mod agent_common;
mod agent_doc;
mod agent_locator;
mod agent_options;
mod prompt_part;

pub use agent_common::*;
pub use agent_doc::*;
pub use agent_locator::*;
pub use prompt_part::*;

// agent_config does not need to be shared beyond module if not test
#[cfg(test)]
pub use agent_options::*;

// endregion: --- Modules
