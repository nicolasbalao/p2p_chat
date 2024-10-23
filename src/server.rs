use std::{io::Error, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    sync::mpsc::Receiver,
};

pub async fn start(port: &str, rx: Receiver<String>) -> Result<(), Error> {
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    println!("Server listening on 0.0.0.0:{}", port);

    loop {
        let (handle, addr) = listener.accept().await?;

        println!("New client: {addr}");

        let (reader, mut writer) = handle.into_split();

        let client_read = tokio::spawn(async move {
            let mut buffer = BufReader::new(reader);
            loop {
                let mut buff = [0; 1024];
                let n = match buffer.read(&mut buff).await {
                    Ok(n) if n == 0 => {
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
                    println!("Peer: {}", text);
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

                writer
                    .write_all(msg.as_bytes())
                    .await
                    .expect("Failed to send message");
            }
        });

        tokio::try_join!(client_write, client_read)?;

        println!("Write and Read task is ended");
    }
}
