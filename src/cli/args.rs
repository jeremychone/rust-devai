use crate::exec::ExecCommand;
use clap::{Parser, Subcommand, command};

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
	/// Initialize the workspace `.aipack/` and the base `~/.aipack-base/` aipack directories
	Init(InitArgs),

	#[command(name = "init-base", about = "Init the ~/.aipack-base only with force update")]
	InitBase,

	#[command(
		about = "Executes the AIPack agent using `aip run demo@craft/code`, or an agent file `aip run path/to/agent.aip`.\n\n\
    Example usage:\n\
    ```sh\n\
    # Run the demo@craft/code AIP agent\n\
    aip run demo@craft/code\n\
    \n\
    # Run the demo@proof main.aip agent and provide a file as input\n\
    aip run demo@proof -f ./README.md\n\
    \n\
    # Run a direct agent file from the local directory\n\
    aip run some/agent.aip\n\
    ```"
	)]
	Run(RunArgs),

	/// List the available aipacks `aip run list` or `aip run list demo@`
	List(ListArgs),

	/// Pack a directory into a .aipack file
	Pack(PackArgs),
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
			// CliCommand::New(_) => false,
			CliCommand::List(_) => false,
			CliCommand::Pack(_) => false,
		}
	}
}

// region:    --- Sub Command Args

/// Arguments for the `run` subcommand
#[derive(Parser, Debug)]
pub struct RunArgs {
	#[clap(help = "The name of the agent, which can be:\n\
- A AIP pack reference:\n\
  `aip run demo@proof`\n\
- Or a direct file:\n\
  `aip run path/to/agent.aip`")]
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

	/// Attempt to open the agent file (for now use VSCode code command)
	#[arg(short = 'o', long = "open")]
	pub open: bool,

	/// Dry mode, takes either 'req' or 'res'
	#[arg(long = "dry", value_parser = ["req", "res"])]
	pub dry_mode: Option<String>,

	/// Non-interactive mode (one-shot execution)
	#[arg(long = "not-interactive", alias = "ni")]
	pub not_interactive: bool,
}

/// Arguments for the `pack` subcommand
#[derive(Parser, Debug)]
pub struct PackArgs {
	/// The directory to pack into a .aipack file
	pub dir_path: String,

	/// Optional destination directory for the .aipack file
	/// If not provided, the .aipack file will be created in the current directory
	#[arg(short = 'o', long = "output")]
	pub output_dir: Option<String>,
}

/// Arguments for the `run` subcommand
#[derive(Parser, Debug)]
pub struct ListArgs {
	/// A complete or partial aipack reference
	/// (optional)
	/// e.g., `jc@coder` or `jc@` or `@coder`
	pub pack_ref: Option<String>,

	/// Open the .aipack file, and the target file if exists.
	/// Note: For now assume vscode `code ...` is installed
	#[arg(short = 'o', long = "open")]
	pub open: bool,
}

/// DISABLED FOR NOW
/// Arguments for the `run` subcommand
#[derive(Parser, Debug)]
pub struct NewArgs {
	pub agent_path: String,

	/// Open the .aipack file, and the target file if exists.
	/// Note: For now assume vscode `code ...` is installed
	#[arg(short = 'o', long = "open")]
	pub open: bool,
}

#[derive(Parser, Debug)]
pub struct InitArgs {
	/// The optional path of were to init the .aipack (relative to current directory)
	/// If not given, aipack will find the closest .aipack/ or create one at current directory
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
			// CliCommand::New(new_args) => ExecCommand::NewCommandAgent(new_args),
			CliCommand::List(list_args) => ExecCommand::List(list_args),
			CliCommand::Pack(pack_args) => ExecCommand::Pack(pack_args),
		}
	}
}

// endregion: --- From CliCommand to ExecCommand
