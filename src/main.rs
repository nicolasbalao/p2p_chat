use std::{env, io::stdin};

use crossterm::style::Stylize;
use tokio::sync::mpsc;
use ui::{clear_screen, print_welcome_message};

mod client;
mod server;
mod ui;
mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut args = env::args();

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
            let input_trimed = input.trim_end();
            let mut input_splited = input_trimed.split_whitespace();
            let command = input_splited.next().unwrap();
            match command {
                "/connect" => {
                    let args = input_splited.next().unwrap();

                    let mut args = args.split(":");

                    let addr = args.next().expect("Failed to get the ip");
                    let port = args.next().expect("Failed to get the port");

                    if let Err(e) = client::connect(addr, port).await {
                        eprintln!("Connection failed: {e}");
                    }

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
