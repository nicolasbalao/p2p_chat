use std::io::Error;

use tokio::{io::AsyncWriteExt, sync::mpsc::Receiver};

pub async fn start(port: &str, mut rx: Receiver<String>) -> Result<(), Error> {
    // Listen on port 3425
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    println!("Server listening on 0.0.0.0:{}", port);

    loop {
        let (handle, addr) = listener.accept().await?;

        println!("New client: {addr}");

        let (mut reader, mut writer) = handle.into_split();

        tokio::spawn(async move {
            let _ = tokio::io::copy(&mut reader, &mut tokio::io::stdout()).await;
        });

        while let Some(msg) = rx.recv().await {
            writer
                .write_all(msg.as_bytes())
                .await
                .expect("Failed to write to socket");
        }

        // read_write(reader, writer, rx).await;
        println!("Connection closed by the client");
    }
}
