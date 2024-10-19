use std::io::Error;

use crate::utils::read_write;

pub async fn start() -> Result<(), Error> {
    // Listen on port 3425
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8989").await?;
    println!("Server listening on 0.0.0.0:8989");

    let (handle, addr) = listener.accept().await?;

    println!("New client: {addr}");

    let (reader, writer) = handle.into_split();

    read_write(reader, writer).await;
    println!("Connection closed by the client");

    Ok(())
}
