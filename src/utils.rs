use std::{
    io::stdout,
    time::{SystemTime, UNIX_EPOCH},
};

use crossterm::{
    cursor, execute,
    style::Stylize,
    terminal::{Clear, ClearType},
};

pub fn print_welcome_message() {
    // Welcome Header
    let header =
        "==========================\n Welcome to the Chat App!\n==========================";

    // Format the header with color
    let colored_header = header.yellow().bold();
    println!("{}", colored_header);

    // Commands section
    let commands = "
Available Commands:
- /connect IP:PORT  -> Connect to a peer
- /exit             -> Exit the chat
";

    // Format the commands with different colors
    let colored_commands = commands.green();
    println!("{}", colored_commands);
}

pub fn clear_screen() {
    execute!(
        stdout(),
        Clear(crossterm::terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .expect("Failed to clear the screen");
}

pub fn get_timestamp() -> String {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    let seconds = since_the_epoch.as_secs();
    let minutes = (seconds / 60) % 60;
    let hours = (seconds / 3600) % 24;

    format!("[{:02}:{:02}]", hours, minutes)
}

// Clear the current input line
pub fn clear_current_input_line() {
    execute!(
        stdout(),
        cursor::MoveUp(1),
        Clear(ClearType::CurrentLine),
        cursor::MoveToColumn(0)
    )
    .unwrap();
}
