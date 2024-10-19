use crate::utils;

pub async fn connect(addr: &str, port: &str) -> Result<(), std::io::Error> {
    let connection = tokio::net::TcpStream::connect(format!("{}:{}", addr, port)).await?;

    println!("Connected with: {}:{}", addr, port);

    let (reader, writer) = connection.into_split();
    utils::read_write(reader, writer).await;

    println!("Connection closed by the server");

    Ok(())
}
