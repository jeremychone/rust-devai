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

	#[command(
		about = "Run a solo agent for a <path> relative to where the devai is run.",
		long_about = "Run a solo agent for a <path> relative to where the devai is run.\n
For convenience, the <path> can be either:\n
  - The target file, e.g., `devai solo ./src/main.rs`
    This will automatically add the '.devai' to run solo as `./src/main.rs.devai`.\n
  - Or the solo file directly, e.g., `./src/main.rs.devai`.\n
IMPORTANT: The path should be at the parent folder of the `.devai/` directory."
	)]
	Solo(SoloArgs),

	/// Create a New Command Agent under `.devai/custom/command-agent/`
	New(NewArgs),

	/// List the available command agents
	List,
}

/// Custom function
impl CliCommand {
	/// Returns true if this CliCommand should be in interative mode.
	///
	/// For now, for all Run and Solo, the interactive is on by default, regardless if it watch.
	pub fn is_interactive(&self) -> bool {
		match self {
			CliCommand::Run(_) => if let CliCommand::Run(run_args) = self {
				run_args.watch
			} else {
				false
			},
			CliCommand::Solo(_) => true,
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
}

/// Arguments for the `solo` subcommand
#[derive(Parser, Debug)]
pub struct SoloArgs {
	/// The solo agent file path or the target file path
	/// - if endsWith `.devai` then it is considered to be the solo agent
	/// - if it does not end with `.devai` then it is considered to be the target file
	///   and therefore the correct `.devai` will be created.
	pub path: String,

	/// Optional watch flag
	#[arg(short = 'w', long = "watch")]
	pub watch: bool,

	/// Verbose mode
	#[arg(short = 'v', long = "verbose")]
	pub verbose: bool,

	/// Attempt to open the solo agent file and the target file (if exists)
	/// (for now use VSCode code command)
	#[arg(short = 'o', long = "open")]
	pub open: bool,

	/// Dry mode, takes either 'req' or 'res'
	#[arg(long = "dry", value_parser = ["req", "res"])]
	pub dry_mode: Option<String>,
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

/// Arguments for the `solo` subcommand
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
			CliCommand::Solo(solo_args) => ExecCommand::RunSoloAgent(solo_args),
			CliCommand::List => ExecCommand::List,
		}
	}
}

// endregion: --- From CliCommand to ExecCommand
