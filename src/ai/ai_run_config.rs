use crate::exec::ExecRunConfig;

#[derive(Debug, Default, Clone)]
pub struct AiRunConfig {
	verbose: bool,
}

impl AiRunConfig {
	pub fn verbose(&self) -> bool {
		self.verbose
	}
}

// region:    --- Froms

impl From<ExecRunConfig> for AiRunConfig {
	fn from(exec_run_config: ExecRunConfig) -> Self {
		Self {
			verbose: exec_run_config.verbose(),
		}
	}
}

impl From<&ExecRunConfig> for AiRunConfig {
	fn from(exec_run_config: &ExecRunConfig) -> Self {
		Self {
			verbose: exec_run_config.verbose(),
		}
	}
}

// endregion: --- Froms
