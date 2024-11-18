use crate::cli::{RunArgs, SoloArgs};
use crate::Result;
use derive_more::derive::From;
use simple_fs::SPath;
use std::sync::Arc;

// region:    --- RunCommandOptions

/// Note: Need to be cloned to able to be part of the RedoCtx
#[derive(Debug, Clone)]
pub struct RunCommandOptions {
	inner: Arc<RunCommandOptionsInner>,
}

#[derive(Debug)]
pub struct RunCommandOptionsInner {
	on_file_globs: Option<Vec<String>>,
	on_inputs: Option<Vec<String>>,

	base_run_options: RunBaseOptions,
}

impl From<RunCommandOptionsInner> for RunCommandOptions {
	fn from(inner: RunCommandOptionsInner) -> Self {
		Self { inner: Arc::new(inner) }
	}
}

/// Getters
impl RunCommandOptions {
	pub fn on_file_globs(&self) -> Option<Vec<&str>> {
		self.inner
			.on_file_globs
			.as_ref()
			.map(|v| v.iter().map(|s| s.as_str()).collect())
	}

	pub fn on_inputs(&self) -> Option<Vec<&str>> {
		self.inner.on_inputs.as_ref().map(|v| v.iter().map(|s| s.as_str()).collect())
	}

	pub fn base_run_config(&self) -> &RunBaseOptions {
		&self.inner.base_run_options
	}
}

/// Constructors
impl RunCommandOptions {
	pub fn new(args: RunArgs) -> Result<Self> {
		// -- Validate the run_args
		if let (Some(_), Some(_)) = (args.on_inputs.as_ref(), args.on_files.as_ref()) {
			return Err("Cannot use both --on-inputs and --on-files".into());
		}

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

		// -- Build the base Options
		let base_run_options = RunBaseOptions {
			watch: args.watch,
			verbose: args.verbose,
			dry_mode,
			open: args.open,
		};

		Ok(RunCommandOptionsInner {
			on_file_globs,
			on_inputs: args.on_inputs,
			base_run_options,
		}
		.into())
	}
}

// endregion: --- RunCommandOptions

// region:    --- RunSoloOptions

/// Note: need to be clonable for redctx
#[derive(Debug, Clone, From)]
pub struct RunSoloOptions {
	inner: Arc<RunSoloOptionsInner>,
}

// inner
#[derive(Debug)]
pub struct RunSoloOptionsInner {
	target_path: SPath,
	base_run_config: RunBaseOptions,
}

impl From<RunSoloOptionsInner> for RunSoloOptions {
	fn from(inner: RunSoloOptionsInner) -> Self {
		Self { inner: Arc::new(inner) }
	}
}

/// Getters
impl RunSoloOptions {
	pub fn target_path(&self) -> &SPath {
		&self.inner.target_path
	}

	pub fn base_run_config(&self) -> &RunBaseOptions {
		&self.inner.base_run_config
	}
}

/// Constructors
impl RunSoloOptions {
	pub fn new(args: SoloArgs, target_path: SPath) -> Result<Self> {
		let dry_mode = parse_dry_mode(args.dry_mode.as_deref());

		let base_run_config = RunBaseOptions {
			watch: args.watch,
			verbose: args.verbose,
			dry_mode,
			open: args.open,
		};

		Ok(RunSoloOptionsInner {
			target_path,
			base_run_config,
		}
		.into())
	}
}

/// For testing only
impl RunSoloOptions {
	#[cfg(test)]
	pub fn from_target_path(path: &str) -> Result<Self> {
		Ok(RunSoloOptionsInner {
			target_path: SPath::new(path)?,
			base_run_config: RunBaseOptions::default(),
		}
		.into())
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

// region:    --- Support

fn parse_dry_mode(dry_mode: Option<&str>) -> DryMode {
	match dry_mode {
		Some("req") => DryMode::Req,
		Some("res") => DryMode::Res,
		_ => DryMode::None,
	}
}

// endregion: --- Support
