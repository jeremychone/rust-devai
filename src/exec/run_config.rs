use crate::agent::get_solo_and_target_path;
use crate::cli::{RunArgs, SoloArgs};
use crate::{Error, Result};
use simple_fs::SPath;

// region:    --- CommandConfig

#[derive(Debug)]
pub struct CommandConfig {
	cmd_agent: String,
	on_file_globs: Option<Vec<String>>,

	base_run_config: BaseRunConfig,
}

impl CommandConfig {
	pub fn cmd_agent(&self) -> &str {
		&self.cmd_agent
	}

	pub fn on_file_globs(&self) -> Option<Vec<&str>> {
		self.on_file_globs.as_ref().map(|v| v.iter().map(|s| s.as_str()).collect())
	}

	pub fn watch(&self) -> bool {
		self.base_run_config.watch
	}

	pub fn verbose(&self) -> bool {
		self.base_run_config.verbose
	}

	pub fn dry_mode(&self) -> DryMode {
		self.base_run_config.dry_mode.clone()
	}
}

impl From<RunArgs> for CommandConfig {
	fn from(args: RunArgs) -> Self {
		// -- When a simple name is provided
		let on_file_globs = if let Some(on_files) = args.on_files {
			let on_files_globs = on_files
				.into_iter()
				.map(|s| {
					// The goal of this logic is to make a simple name into a wider glob so that the user does not have to specify the exact file name.
					// TODO: This branch can be improved to handle any absolute or relative path.
					if s.contains('*') || s.starts_with("./") || s.starts_with("/") {
						s
					} else {
						format!("**/{s}")
					}
				})
				.collect::<Vec<String>>();
			Some(on_files_globs)
		} else {
			None
		};

		let dry_mode = match args.dry_mode.as_deref() {
			Some("req") => DryMode::Req,
			Some("res") => DryMode::Res,
			_ => DryMode::None,
		};

		let base_run_config = BaseRunConfig {
			watch: args.watch,
			verbose: args.verbose,
			dry_mode,
		};

		Self {
			cmd_agent: args.cmd_agent_name,
			on_file_globs,
			base_run_config,
		}
	}
}

// endregion: --- CommandConfig

// region:    --- SoloConfig

#[derive(Debug)]
pub struct SoloConfig {
	solo_path: SPath,
	target_path: SPath,
	base_run_config: BaseRunConfig,
}

/// Getters
impl SoloConfig {
	pub fn solo_path(&self) -> &SPath {
		&self.solo_path
	}

	pub fn target_path(&self) -> &SPath {
		&self.target_path
	}

	pub fn watch(&self) -> bool {
		self.base_run_config.watch
	}
	pub fn verbose(&self) -> bool {
		self.base_run_config.verbose
	}
}

impl TryFrom<SoloArgs> for SoloConfig {
	type Error = Error;

	fn try_from(args: SoloArgs) -> Result<Self> {
		let (solo_path, target_path) = get_solo_and_target_path(args.path)?;

		let base_run_config = BaseRunConfig {
			watch: args.watch,
			verbose: args.verbose,
			dry_mode: DryMode::None, // Not Supported for now
		};

		Ok(Self {
			solo_path,
			target_path,
			base_run_config,
		})
	}
}

// endregion: --- SoloConfig

// region:    --- Common

/// The Dry mode of the content.
///
/// > Note: Might want to move this out of the exec sub module as it is used in ai one (code-clean)
#[derive(Debug, Clone)]
pub enum DryMode {
	Req,
	Res,
	None, // not dry mode
}

#[derive(Debug)]
pub struct BaseRunConfig {
	watch: bool,
	verbose: bool,
	dry_mode: DryMode,
}

// endregion: --- Common
