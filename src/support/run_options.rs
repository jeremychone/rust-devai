use crate::agent::Agent;
use crate::ai::Literals;
use crate::cli::{RunArgs, SoloArgs};
use crate::support::DirContext;
use crate::Result;
use simple_fs::SPath;
use std::path::Path;

// region:    --- RunCommandOptions

#[derive(Debug)]
pub struct RunCommandOptions {
	on_file_globs: Option<Vec<String>>,

	base_run_options: RunBaseOptions,
}

/// Getters
impl RunCommandOptions {
	pub fn on_file_globs(&self) -> Option<Vec<&str>> {
		self.on_file_globs.as_ref().map(|v| v.iter().map(|s| s.as_str()).collect())
	}

	pub fn base_run_config(&self) -> &RunBaseOptions {
		&self.base_run_options
	}
}

/// Constructors
impl RunCommandOptions {
	pub fn new(args: RunArgs, dir_context: &DirContext, agent: &Agent) -> Result<Self> {
		// -- Refine the globs
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

		// -- Parse dry_mode
		let dry_mode = parse_dry_mode(args.dry_mode.as_deref());

		// -- Build the literal
		let literals = build_literals(dir_context, agent)?;

		// -- Build the base Options
		let base_run_options = RunBaseOptions {
			watch: args.watch,
			verbose: args.verbose,
			dry_mode,
			open: args.open,
			literals,
		};

		Ok(Self {
			on_file_globs,
			base_run_options,
		})
	}
}

// endregion: --- RunCommandOptions

// region:    --- RunSoloOptions

#[derive(Debug)]
pub struct RunSoloOptions {
	target_path: SPath,
	base_run_config: RunBaseOptions,
}

/// Getters
impl RunSoloOptions {
	pub fn target_path(&self) -> &SPath {
		&self.target_path
	}

	pub fn base_run_config(&self) -> &RunBaseOptions {
		&self.base_run_config
	}
}

/// Constructors
impl RunSoloOptions {
	pub fn new(args: SoloArgs, dir_context: &DirContext, agent: &Agent, target_path: SPath) -> Result<Self> {
		let dry_mode = parse_dry_mode(args.dry_mode.as_deref());

		// -- Build the literal
		let literals = build_literals(dir_context, agent)?;

		let base_run_config = RunBaseOptions {
			watch: args.watch,
			verbose: args.verbose,
			dry_mode,
			open: args.open,
			literals,
		};

		Ok(Self {
			target_path,
			base_run_config,
		})
	}
}

/// For testing only
impl RunSoloOptions {
	#[cfg(test)]
	pub fn from_target_path(path: &str) -> Result<Self> {
		Ok(Self {
			target_path: SPath::new(path)?,
			base_run_config: RunBaseOptions::default(),
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
	literals: Literals,
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

	pub fn literals_as_strs(&self) -> Vec<(&str, &str)> {
		self.literals.as_strs()
	}
}

// endregion: --- Common

// region:    --- Support

fn build_literals(dir_context: &DirContext, agent: &Agent) -> Result<Literals> {
	let mut literals = Literals::default();

	let agent_path = agent.file_path();
	let agent_dir = Path::new(agent.file_path())
		.parent()
		.ok_or_else(|| format!("Agent with path '{}' does not have a parent path", agent.file_path()))?
		.to_str()
		.ok_or("File path is not utf8")?;

	let devai_dir = dir_context.devai_dir();

	literals.append("$DEVAI_AGENT_DIR", agent_dir);
	literals.append("$DEVAI_AGENT_PATH", agent_path);
	literals.append("$DEVAI_DIR", devai_dir.to_str());
	// TOOD: Need to have a better strategy when parent is none
	literals.append(
		"$DEVAI_PARENT_DIR",
		devai_dir.parent().as_ref().map(|p| p.to_str()).unwrap_or_else(|| ""),
	);

	Ok(literals)
}

fn parse_dry_mode(dry_mode: Option<&str>) -> DryMode {
	match dry_mode {
		Some("req") => DryMode::Req,
		Some("res") => DryMode::Res,
		_ => DryMode::None,
	}
}

// endregion: --- Support
