use std::{env, io::stdin};

use tokio::sync::mpsc;

mod client;
mod server;
mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut args = env::args();

    let port = args.nth(1).expect("Failed to read port arg");

    let (tx, rx) = mpsc::channel(100);

    tokio::spawn(async move {
        if let Err(e) = server::start(&port, rx).await {
            eprintln!("Server error: {e}");
        }
    });

    let mut input = String::new();
    let stdin = stdin();

    println!("Command: /connect: IP:PORT");
    loop {
        stdin
            .read_line(&mut input)
            .expect("Failed to read stdin input");

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
            }
            _ => {
                tx.send(input.to_string())
                    .await
                    .expect("Failed to send message in channel");
            }
        }
    }
}
