use crate::exec::ExecCommand;
use clap::{command, Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
	/// Subcommands
	#[command(subcommand)]
	pub cmd: CliCommand,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
	/// Initialize the `.devai/` folder with the base setting files. Any file that already exists will not be touched.
	Init(InitArgs),

	#[command(name = "init-base", about = "Init the ~/.devai-base")]
	InitBase,

	/// Executes the Command Agent `<name>` based on its name or short name.
	///
	/// The `<name>` is relative to the `.devai/[default|custom]/command-agent/<name>.devai`
	///
	/// for example `devai run proof-comments` or `devai run pc` will match:
	/// either `.devai/custom/command-agent/proof-comments.devai`
	/// and if not found will look in `.devai/default/command-agent/proof-comments.devai`
	Run(RunArgs),

	/// Create a New Command Agent under `.devai/custom/command-agent/`
	New(NewArgs),

	/// List the available command agents
	List,
}

/// Custom function
impl CliCommand {
	/// Returns true if this CliCommand should be in interative mode.
	///
	/// For now, for all Run, the interactive is on by default, regardless if it watch.
	pub fn is_interactive(&self) -> bool {
		match self {
			CliCommand::Run(run_args) => !run_args.not_interactive,
			CliCommand::Init(_) => false,
			CliCommand::InitBase => false,
			CliCommand::New(_) => false,
			CliCommand::List => false,
		}
	}
}

// region:    --- Sub Command Args

/// Arguments for the `run` subcommand
#[derive(Parser, Debug)]
pub struct RunArgs {
	/// The name of the Command Agent to execute, required.
	/// This should be the name of the markdown file under `.devai/customs` or `.devai/defaults` (without extension),
	/// or the filename initial `proof-comments.md` will match to `proof-comments` or `pc`
	pub cmd_agent_name: String,

	/// Optional input, allowing multiple input
	/// NOTE: CANNOT be combined with -f/--on-files
	#[arg(short = 'i', long = "input")]
	pub on_inputs: Option<Vec<String>>,

	/// Optional file parameter, allowing multiple files
	/// NOTE: CANNOT be combined with -i/--input
	#[arg(short = 'f', long = "on-files")]
	pub on_files: Option<Vec<String>>,

	/// Optional watch flag
	#[arg(short = 'w', long = "watch")]
	pub watch: bool,

	/// Verbose mode
	#[arg(short = 'v', long = "verbose")]
	pub verbose: bool,

	/// Attempt to open the command agent file (for now use VSCode code command)
	#[arg(short = 'o', long = "open")]
	pub open: bool,

	/// Dry mode, takes either 'req' or 'res'
	#[arg(long = "dry", value_parser = ["req", "res"])]
	pub dry_mode: Option<String>,

	/// Non-interactive mode (one-shot execution)
	#[arg(long = "not-interactive", alias = "ni")]
	pub not_interactive: bool,
}

/// Arguments for the `run` subcommand
#[derive(Parser, Debug)]
pub struct NewArgs {
	/// The command agent name which will be created under
	/// e.g., `devai new my-cool-agent`
	///        will create `.devai/custom/command-agent/my-cool-agent.devai`
	pub agent_path: String,

	/// Open the .devai file, and the target file if exists.
	/// Note: For now assume vscode `code ...` is installed
	#[arg(short = 'o', long = "open")]
	pub open: bool,
}

#[derive(Parser, Debug)]
pub struct InitArgs {
	/// The optional path of were to init the .devai (relative to current directory)
	/// If not given, devai will find the closest .devai/ or create one at current directory
	pub path: Option<String>,
}

// endregion: --- Sub Command Args

// region:    --- From CliCommand to ExecCommand

impl From<CliCommand> for ExecCommand {
	fn from(cli_cmd: CliCommand) -> Self {
		match cli_cmd {
			CliCommand::Init(init_args) => ExecCommand::Init(init_args),
			CliCommand::InitBase => ExecCommand::InitBase,
			CliCommand::Run(run_args) => ExecCommand::RunCommandAgent(run_args),
			CliCommand::New(new_args) => ExecCommand::NewCommandAgent(new_args),
			CliCommand::List => ExecCommand::List,
		}
	}
}

// endregion: --- From CliCommand to ExecCommand
