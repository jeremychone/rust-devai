// region:    --- Modules

mod agent;
mod ai;
mod cli;
mod error;
mod script;
mod support;
mod tmpl;
mod types;

use crate::agent::{find_agent, init_agent_files};
use crate::ai::run_agent;
use crate::cli::{AppArgs, CmdConfig};
use crate::support::cred::get_or_prompt_api_key;
use crate::types::FileRef;
use clap::Parser;
pub use error::{Error, Result};
use serde_json::json;
use simple_fs::list_files;

// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
	// -- command arguments
	let args = AppArgs::parse(); // will fail early, but that's okay.
	let cmd_config = CmdConfig::from(args);

	// -- Init the default agent files
	init_agent_files();

	// -- get AI client and agent
	let client = ai::get_genai_client()?;
	let agent = find_agent(cmd_config.cmd_agent())?;

	// -- Execute the command
	let on_file_globs = cmd_config.on_file_globs();
	// if we have the on_file_globs, they become the items
	if let Some(on_file_globs) = on_file_globs {
		let files = list_files("./", Some(&on_file_globs), None)?;
		for sfile in files {
			let file_ref = FileRef::from(sfile);
			let scope_value = json!({
				"item": file_ref
			});
			run_agent(&client, &agent, Some(scope_value)).await?;
		}
	} else {
		run_agent(&client, &agent, None).await?;
	}

	Ok(())
}
