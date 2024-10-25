use std::{
    io::{stdout, Write},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crossterm::{
    cursor, execute,
    style::Stylize,
    terminal::{Clear, ClearType},
};
use tokio::time::sleep;

pub fn print_welcome_message(port: &str) {
    // Welcome Header
    let header = r#"

 ▄▄▄▄▄▄▄ ▄▄   ▄▄ ▄▄▄▄▄▄ ▄▄▄▄▄▄▄    ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄ 
█       █  █ █  █      █       █  █       █       █       █
█       █  █▄█  █  ▄   █▄     ▄█  █   ▄   █    ▄  █    ▄  █
█     ▄▄█       █ █▄█  █ █   █    █  █▄█  █   █▄█ █   █▄█ █
█    █  █   ▄   █      █ █   █    █       █    ▄▄▄█    ▄▄▄█
█    █▄▄█  █ █  █  ▄   █ █   █    █   ▄   █   █   █   █    
█▄▄▄▄▄▄▄█▄▄█ █▄▄█▄█ █▄▄█ █▄▄▄█    █▄▄█ █▄▄█▄▄▄█   █▄▄▄█    

    "#;

    // Format the header with color
    let colored_header = header.yellow().bold();
    println!("{}", colored_header);

    let infos = format!("Server listening on 0.0.0.0:{}", port).blue();
    println!("{}", infos);
    // Commands sectio
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

// Fancy ASCII art for chat banner
fn print_banner() {
    let banner = r#"

  ___               ___  
 (o o)             (o o) 
(  V  ) Chat room (  V  )
--m-m---------------m-m--

    "#;

    println!("{}", banner.blue().bold());
    println!("{}", "  Welcome to the Chat App!  ".yellow().bold());
}

// Simulate connecting process
async fn simulate_connecting(peer_addr: &str) {
    print!("Connecting to {} ", peer_addr.green().bold());
    stdout().flush().unwrap();

    for _ in 0..5 {
        print!(".");
        stdout().flush().unwrap();
        sleep(Duration::from_millis(500)).await; // Simulating delay
    }

    println!("{}", "\nConnected successfully!".green().bold());
}

// Function to display fancy chat start message
pub async fn start_chat_screen(peer_addr: &str) {
    clear_screen();
    print_banner();
    simulate_connecting(peer_addr).await;

    println!("\n{}", "-----------------------------------".yellow());
    println!("You are now chatting with: {}", peer_addr.cyan().bold());
    println!("Type /exit to end the chat.");
    println!("{}", "-----------------------------------".yellow());
    println!();
}
