use crate::agent::get_solo_and_target_path;
use crate::cli::{RunArgs, SoloArgs};
use crate::{Error, Result};
use simple_fs::SPath;

// region:    --- RunCommandOptions

#[derive(Debug)]
pub struct RunCommandOptions {
	cmd_agent: String,
	on_file_globs: Option<Vec<String>>,

	base_run_config: RunBaseOptions,
}

impl RunCommandOptions {
	pub fn cmd_agent(&self) -> &str {
		&self.cmd_agent
	}

	pub fn on_file_globs(&self) -> Option<Vec<&str>> {
		self.on_file_globs.as_ref().map(|v| v.iter().map(|s| s.as_str()).collect())
	}

	pub fn base_run_config(&self) -> &RunBaseOptions {
		&self.base_run_config
	}
}

impl From<RunArgs> for RunCommandOptions {
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

		let dry_mode = parse_dry_mode(args.dry_mode.as_deref());

		let base_run_config = RunBaseOptions {
			watch: args.watch,
			verbose: args.verbose,
			dry_mode,
			open: args.open,
		};

		Self {
			cmd_agent: args.cmd_agent_name,
			on_file_globs,
			base_run_config,
		}
	}
}

// endregion: --- RunCommandOptions

// region:    --- RunSoloOptions

#[derive(Debug)]
pub struct RunSoloOptions {
	solo_path: SPath,
	target_path: SPath,
	base_run_config: RunBaseOptions,
}

/// Getters
impl RunSoloOptions {
	pub fn solo_path(&self) -> &SPath {
		&self.solo_path
	}

	pub fn target_path(&self) -> &SPath {
		&self.target_path
	}

	pub fn base_run_config(&self) -> &RunBaseOptions {
		&self.base_run_config
	}
}

impl RunSoloOptions {
	#[cfg(test)]
	pub fn from_path(path: &str) -> Result<Self> {
		let (solo_path, target_path) = get_solo_and_target_path(path)?;
		Ok(Self {
			solo_path,
			target_path,
			base_run_config: RunBaseOptions::default(),
		})
	}
}

impl TryFrom<SoloArgs> for RunSoloOptions {
	type Error = Error;

	fn try_from(args: SoloArgs) -> Result<Self> {
		let (solo_path, target_path) = get_solo_and_target_path(args.path)?;

		let dry_mode = parse_dry_mode(args.dry_mode.as_deref());

		let base_run_config = RunBaseOptions {
			watch: args.watch,
			verbose: args.verbose,
			dry_mode,
			open: args.open,
		};

		Ok(Self {
			solo_path,
			target_path,
			base_run_config,
		})
	}
}

// endregion: --- RunSoloOptions

// region:    --- Common

/// The Dry mode of the content.
///
/// > Note: Might want to move this out of the exec sub module as it is used in ai one (code-clean)
#[derive(Debug, Clone, Default)]
pub enum DryMode {
	Req,
	Res,
	#[default]
	None, // not dry mode
}

#[derive(Debug, Clone, Default)]
pub struct RunBaseOptions {
	watch: bool,
	verbose: bool,
	dry_mode: DryMode,
	open: bool,
}

impl RunBaseOptions {
	pub fn watch(&self) -> bool {
		self.watch
	}

	pub fn verbose(&self) -> bool {
		self.verbose
	}

	pub fn dry_mode(&self) -> &DryMode {
		&self.dry_mode
	}

	pub fn open(&self) -> bool {
		self.open
	}
}

// endregion: --- Common

// region:    --- Section

fn parse_dry_mode(dry_mode: Option<&str>) -> DryMode {
	match dry_mode {
		Some("req") => DryMode::Req,
		Some("res") => DryMode::Res,
		_ => DryMode::None,
	}
}

// endregion: --- Section
