use std::{io::Error, sync::Arc};

use tokio::{io::AsyncWriteExt, select, sync::mpsc::Receiver};

pub async fn start(port: &str, rx: Receiver<String>) -> Result<(), Error> {
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    println!("Server listening on 0.0.0.0:{}", port);

    loop {
        let (handle, addr) = listener.accept().await?;

        println!("New client: {addr}");

        let (mut reader, mut writer) = handle.into_split();

        let client_read = tokio::spawn(async move {
            let _ = tokio::io::copy(&mut reader, &mut tokio::io::stdout()).await;
        });

        let rx_clone = Arc::clone(&rx);

        let client_write = tokio::spawn(async move {
            let mut rx = rx_clone.lock().await;

            while let Some(msg) = rx.recv().await {
                writer
                    .write_all(msg.as_bytes())
                    .await
                    .expect("Failed to send message");
            }
        });

        select! {
            _ = client_read => {},
            _ = client_write => {}
        }
        println!("Connection closed by the client");
    }
}
