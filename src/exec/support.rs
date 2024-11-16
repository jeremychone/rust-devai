use crate::hub::get_hub;
use crate::Error;
use std::path::Path;
use std::process::Command;

/// Attempt to open a path via vscode
/// NOTE: VSCode will do the right thing when the user have multiple vscode open
///       by opening the path in the corresponding workspace.
pub async fn open_vscode(path: impl AsRef<Path>) {
	let path = path.as_ref();

	let output = Command::new("code")
		.arg(path)
		.output()
		.expect("Failed to execute VSCode 'code' command");

	if !output.status.success() {
		let msg = format!("Error opening VSCode: {}", String::from_utf8_lossy(&output.stderr));
		get_hub().publish(Error::Custom(msg.to_string())).await;
	}
}
