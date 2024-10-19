use std::io::stdin;

mod client;
mod server;
mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Read args client / server

    let args = std::env::args().collect::<Vec<String>>();

    let mode = &args[1];

    match mode.as_str() {
        "client" => {
            let mut input = String::new();
            let stdin = stdin();
            println!("IP:PORT: ");
            stdin
                .read_line(&mut input)
                .expect("Failed to read stdin input");

            let input = input.trim_end();
            if !input.is_empty() {
                let input_splited: Vec<&str> = input.split(":").collect();

                let addr = input_splited[0];
                let port = input_splited[1];

                client::connect(addr, port).await?
            } else {
                client::connect("0.0.0.0", "8989").await?;
            }
        }
        "server" => {
            server::start().await?;
        }
        _ => {
            println!("Invalid mode");
        }
    }

    Ok(())
}
