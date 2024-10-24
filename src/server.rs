use std::{io::Error, sync::Arc};

use crossterm::style::Stylize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    sync::mpsc::Receiver,
};

use crate::utils::{clear_current_input_line, clear_screen, get_timestamp, print_welcome_message};

pub async fn start(port: &str, rx: Receiver<String>) -> Result<(), Error> {
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    let server_started_msg = format!("Server started on: 0.0.0.0:{}", port)
        .blue()
        .italic();
    println!("{}", server_started_msg);

    // REF
    let mut nb_connection = 0;

    loop {
        if nb_connection != 0 {
            clear_screen();
            print_welcome_message();
        }
        let (handle, addr) = listener.accept().await?;

        println!("New client: {addr}");

        let (reader, mut writer) = handle.into_split();

        let client_read = tokio::spawn(async move {
            let mut buffer = BufReader::new(reader);
            loop {
                let mut buff = [0; 1024];
                let n = match buffer.read(&mut buff).await {
                    Ok(0) => {
                        // If we read 0 bytes, that means the connection is closed
                        println!("Connection closed by peer");
                        println!("/exit for end the discussion");
                        break;
                    }
                    Ok(n) => n, // `n` is the number of bytes read
                    Err(e) => {
                        println!("Failed to read from socket: {:?}", e);
                        break;
                    }
                };

                if let Ok(text) = std::str::from_utf8(&buff[..n]) {
                    let text = text.trim_end();
                    let timestamp = get_timestamp();
                    println!("{} {}: {}", timestamp.blue(), "Peer".green(), text);
                } else {
                    println!("Received non-UTF8 data");
                }
            }
        });

        let rx_clone = Arc::clone(&rx);

        let client_write = tokio::spawn(async move {
            let mut rx = rx_clone.lock().await;

            while let Some(msg) = rx.recv().await {
                if &msg == "/exit\n" {
                    println!("Exit the discussion");
                    writer.shutdown().await.expect("Failed to shutdown  writer");
                    break;
                }

                // Clear input
                clear_current_input_line();

                // Format input
                let timestamp = get_timestamp();
                println!(
                    "{} {}: {}",
                    timestamp.blue(),
                    "You".yellow().bold(),
                    msg.trim()
                );

                writer
                    .write_all(msg.as_bytes())
                    .await
                    .expect("Failed to send message");
            }
        });

        tokio::try_join!(client_write, client_read)?;
        println!("Chat ended");
        nb_connection += 1;
    }
}
