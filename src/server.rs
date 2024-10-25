use std::{io::Error, sync::Arc};

use crossterm::style::Stylize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    sync::{mpsc::Receiver, Mutex},
};

use crate::utils::{
    clear_current_input_line, clear_screen, get_timestamp, print_welcome_message, start_chat_screen,
};

pub async fn start(port: &str, rx: Receiver<String>) -> Result<(), Error> {
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    let mut is_first_connection = true;

    loop {
        if !is_first_connection {
            clear_screen();
            print_welcome_message(&port);
        }
        let (handle, addr) = listener.accept().await?;

        let connection_closed = Arc::new(Mutex::new(false));

        start_chat_screen(&addr.to_string()).await;

        let (reader, mut writer) = handle.into_split();

        let connection_closed_read = connection_closed.clone();
        let read_task = tokio::spawn(async move {
            let mut buffer = BufReader::new(reader);
            loop {
                let mut buff = [0; 1024];
                let n = match buffer.read(&mut buff).await {
                    Ok(0) => {
                        // If we read 0 bytes, that means the connection is closed
                        let connection_closed_msg = "Connection closed by the peer".red().bold();
                        let mut connection_closed = connection_closed_read.lock().await;

                        *connection_closed = true;

                        println!("{}", connection_closed_msg);
                        break;
                    }
                    Ok(n) => n,
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

        let connection_closed_write = connection_closed.clone();
        let write_task = tokio::spawn(async move {
            let mut rx = rx_clone.lock().await;

            while let Some(msg) = rx.recv().await {
                if &msg == "/exit\n" {
                    println!("Exit the discussion");
                    if !*connection_closed_write.lock().await {
                        writer.shutdown().await.expect("Failed to shutdown  writer");
                    }
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

                if let Err(e) = writer.write_all(msg.as_bytes()).await {
                    // For keeping the user into the chat room without error
                    // when the peer is disconnected
                    if e.kind() != std::io::ErrorKind::BrokenPipe {
                        println!("Error while sending message: {}", e);
                    }
                }
            }
        });

        tokio::try_join!(write_task, read_task)?;
        is_first_connection = false;
    }
}
