use std::{
    env::{self, Args},
    io::stdin,
    net::SocketAddrV4,
};

use crossterm::style::Stylize;
use tokio::sync::mpsc;
use ui::{clear_screen, print_welcome_message};

mod client;
mod server;
mod ui;
mod utils;

#[tokio::main]
async fn main() {
    let args = env::args();
    if let Err(e) = run(args).await {
        let msg = format!("Error: {}", e).red();
        eprintln!("{}", msg);
        std::process::exit(1);
    }
}

async fn run(mut args: Args) -> std::io::Result<()> {
    let port = args.nth(1).expect("Failed to read port arg");
    let port_clone = port.clone();

    let (tx, rx) = mpsc::channel(100);

    tokio::spawn(async move {
        if let Err(e) = server::start(&port, rx).await {
            eprintln!("Server error: {e}");
        }
    });

    let mut input = String::new();
    let stdin = stdin();

    // Welcome message
    print_welcome_message(&port_clone);

    loop {
        input.clear();
        stdin
            .read_line(&mut input)
            .expect("Failed to read stdin input");

        if input.starts_with("/") {
            let (command, args) = prepare_input(&input);
            match command {
                "/connect" => {
                    let args = match args {
                        Some(args) => args,
                        None => {
                            let msg = "Invalide syntax => /connect IP:PORT".red();
                            eprintln!("{}", msg);
                            continue;
                        }
                    };

                    let peer_socker_addr = match args.parse::<SocketAddrV4>() {
                        Ok(s) => s,
                        Err(e) => {
                            let error_msg = format!("Error: {}", e).red();
                            eprintln!("{}", error_msg);
                            continue;
                        }
                    };

                    if let Err(e) = client::connect(peer_socker_addr).await {
                        eprintln!("Connection failed: {e}");
                    };

                    clear_screen();
                    print_welcome_message(&port_clone);
                }
                // REF this
                "/exit" => tx
                    .send(input.clone())
                    .await
                    .expect("Failed to send message in stdin channel"),
                _ => {
                    let unknown_command = format!("Command {} is unknown", command).red();
                    println!("{}", unknown_command);
                }
            }
        } else {
            tx.send(input.clone())
                .await
                .expect("Failed to send message in channel");
        }
    }
}

fn prepare_input(input: &str) -> (&str, Option<&str>) {
    let input_trimed = input.trim_end();
    let mut input_splited = input_trimed.split_whitespace();
    let command = input_splited.next().expect("Command not found");
    let args = input_splited.next();

    (command, args)
}
