use std::net::SocketAddrV4;

use crate::{
    ui::start_chat_screen,
    utils::{handle_chat, StdinMessageSource},
};

pub async fn connect(peer_socker_addr: SocketAddrV4) -> Result<(), std::io::Error> {
    let connection = tokio::net::TcpStream::connect(peer_socker_addr).await?;

    println!(
        "Connected with: {}:{}",
        peer_socker_addr.ip(),
        peer_socker_addr.port()
    );
    start_chat_screen(&format!(
        "{}:{}",
        peer_socker_addr.ip(),
        peer_socker_addr.port()
    ))
    .await;

    let (reader, writer) = connection.into_split();

    let stdin = StdinMessageSource::new();
    handle_chat(reader, writer, stdin).await;

    Ok(())
}
