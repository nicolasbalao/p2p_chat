use std::sync::Arc;

use tokio::{select, sync::Mutex};

use crate::{
    ui::start_chat_screen,
    utils::{read_from_peer, write_to_peer, StdinMessageSource},
};

pub async fn connect(addr: &str, port: &str) -> Result<(), std::io::Error> {
    let connection = tokio::net::TcpStream::connect(format!("{}:{}", addr, port)).await?;

    println!("Connected with: {}:{}", addr, port);
    start_chat_screen(&format!("{}:{}", addr, port)).await;

    let (reader, writer) = connection.into_split();

    let connection_closed = Arc::new(Mutex::new(false));

    let read_connection_closed = connection_closed.clone();
    let read_task = tokio::spawn(async move {
        read_from_peer(reader, read_connection_closed).await;
    });

    let write_connection_closed = connection_closed.clone();
    let write_task = tokio::spawn(async move {
        let stdin = StdinMessageSource::new();
        write_to_peer(writer, stdin, write_connection_closed).await
    });

    select! {
        _ = write_task => {
            read_task.abort();
        }
    }

    Ok(())
}
