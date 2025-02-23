use crate::Error;
use crate::hub::get_hub;
use std::path::Path;
use std::process::Command;

/// Attempt to open a path via vscode
/// NOTE: VSCode will do the right thing when the user have multiple vscode open
///       by opening the path in the corresponding workspace.
pub async fn open_vscode(path: impl AsRef<Path>) {
	let path = path.as_ref();

	let output = if cfg!(target_os = "windows") {
		Command::new("cmd")
			// for path.to_str().unwrap..., should never happen, but should never crash either
			.args(["/C", "code", path.to_str().unwrap_or_default()])
			.output()
	} else {
		Command::new("code").arg(path).output()
	};

	match output {
		Ok(output) if output.status.success() => {}
		Ok(output) => {
			let msg = format!(
				"Error opening VSCode:\nstdout: {}\nstderr: {}",
				String::from_utf8_lossy(&output.stdout),
				String::from_utf8_lossy(&output.stderr)
			);
			get_hub().publish(Error::Custom(msg)).await;
		}
		Err(e) => {
			let msg = format!("Failed to execute VSCode command: {}", e);
			get_hub().publish(Error::Custom(msg)).await;
		}
	}
}
