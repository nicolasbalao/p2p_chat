use crate::{
    ui::start_chat_screen,
    utils::{handle_chat, StdinMessageSource},
};

pub async fn connect(addr: &str, port: &str) -> Result<(), std::io::Error> {
    let connection = tokio::net::TcpStream::connect(format!("{}:{}", addr, port)).await?;

    println!("Connected with: {}:{}", addr, port);
    start_chat_screen(&format!("{}:{}", addr, port)).await;

    let (reader, writer) = connection.into_split();

    let stdin = StdinMessageSource::new();
    handle_chat(reader, writer, stdin).await;

    Ok(())
}
