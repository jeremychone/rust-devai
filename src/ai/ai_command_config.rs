use crate::ai::AiSoloConfig;
use crate::exec::{CommandConfig, DryMode};

#[derive(Debug, Clone)]
pub struct AiCommandConfig {
	verbose: bool,
	dry_mode: DryMode,
}

impl Default for AiCommandConfig {
	fn default() -> Self {
		Self {
			verbose: false,
			dry_mode: DryMode::None,
		}
	}
}

impl AiCommandConfig {
	pub fn verbose(&self) -> bool {
		self.verbose
	}
	pub fn dry_mode(&self) -> DryMode {
		self.dry_mode.clone()
	}
}

// region:    --- Froms

impl From<CommandConfig> for AiCommandConfig {
	fn from(exec_run_config: CommandConfig) -> Self {
		Self {
			verbose: exec_run_config.verbose(),
			dry_mode: exec_run_config.dry_mode(),
		}
	}
}

impl From<&CommandConfig> for AiCommandConfig {
	fn from(exec_run_config: &CommandConfig) -> Self {
		Self {
			verbose: exec_run_config.verbose(),
			dry_mode: exec_run_config.dry_mode(),
		}
	}
}

impl From<&AiSoloConfig> for AiCommandConfig {
	fn from(value: &AiSoloConfig) -> Self {
		Self {
			verbose: value.verbose(),
			dry_mode: DryMode::None,
		}
	}
}
// endregion: --- Froms
