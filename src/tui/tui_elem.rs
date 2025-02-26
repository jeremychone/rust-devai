use crate::dir_context::PackDir;
use crossterm::{
	cursor::{MoveToColumn, MoveToNextLine},
	execute,
	style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
	terminal::{Clear, ClearType},
};
use std::{
	collections::HashSet,
	io::{Stdout, stdout},
};

// region:    --- Pack List

#[allow(unused_must_use)] // TODO: need to remove and make this function return error
pub fn print_pack_list(pack_dirs: &[PackDir]) {
	let mut stdout = stdout();

	let mut width = 0;
	for pack_dir in pack_dirs.iter() {
		width = width.max(pack_dir.namespace.len() + pack_dir.name.len());
	}
	width += 5;

	let mut existing_set: HashSet<String> = HashSet::new();

	// (active, pack_ref, pretty_path)
	let data: Vec<(bool, String, String)> = pack_dirs
		.iter()
		.map(|p| {
			let pack_ref = p.to_string();
			let active = if existing_set.contains(&pack_ref) {
				false
			} else {
				existing_set.insert(pack_ref.to_string());
				true
			};
			(active, pack_ref, p.pretty_path())
		})
		.collect::<Vec<_>>();

	execute!(stdout, Print("\nListing all available aipacks:\n\n"));

	for (active, name, path) in data.iter() {
		let (bullet, weight_ref, weight_path) = if *active {
			("•", Attribute::Bold, Attribute::Reset)
		} else {
			("-", Attribute::Dim, Attribute::Dim)
		};
		execute!(
			stdout,
			SetAttribute(weight_ref),
			Print(format!("{bullet} {:<width$}", name)),
			ResetColor,
			SetAttribute(weight_path),
			Print(format!("- {}\n", path)),
			ResetColor,
			SetAttribute(Attribute::Reset)
		);
	}
}

// endregion: --- Pack List

// region:    --- Bottom Bar

pub fn print_bottom_bar() {
	let mut stdout = stdout();
	// TODO: Need to handler error
	let _ = execute!(
		stdout,
		Clear(ClearType::CurrentLine), // Clear the current line completely
		MoveToNextLine(2),             // Move the cursor to the next line (down one line)
		Print("\n"),
		MoveToColumn(0), // Move the cursor to the beginning (column 0) of the new line
	);

	term_key_comp(&mut stdout, "r", "Replay");

	let _ = execute!(stdout, Print("  "),);

	term_key_comp(&mut stdout, "a", "Open Agent");

	let _ = execute!(stdout, Print("  "),);

	term_key_comp(&mut stdout, "q", "Quit");

	let _ = execute!(stdout, Print("\n"));
}

/// Return a `[ k ] name` term component in crossterm Commans
pub fn term_key_comp(stdout: &mut Stdout, key: &str, name: &str) {
	let _ = execute!(
		stdout,
		Print("[ "),
		SetForegroundColor(Color::Blue),
		Print(key),
		ResetColor,
		Print(" ]"),
		Print(": "),
		Print(name)
	);
}

// endregion: --- Bottom Bar
