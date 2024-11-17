use crossterm::{
	cursor::{MoveToColumn, MoveToNextLine},
	execute,
	style::{Color, Print, ResetColor, SetForegroundColor},
	terminal::{Clear, ClearType},
};
use std::io::{stdout, Stdout};

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

	term_key_comp(&mut stdout, "r", "Redo");

	let _ = execute!(stdout, Print("   |   "),);

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
