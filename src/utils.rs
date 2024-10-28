use std::sync::Arc;

use async_trait::async_trait;
use crossterm::style::Stylize;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader},
    select,
    sync::{mpsc::Receiver, Mutex},
};

use crate::ui::{clear_current_input_line, get_timestamp};

pub async fn handle_chat<R, W, S>(reader: R, writer: W, writer_source: S)
where
    R: AsyncRead + Send + Unpin + 'static,
    W: AsyncWriteExt + Send + Unpin + 'static,
    S: MessageSource + Send + Unpin + 'static,
{
    let connection_closed = Arc::new(Mutex::new(false));

    let connection_closed_read = connection_closed.clone();
    let read_task = tokio::spawn(async move {
        read_from_peer(reader, connection_closed_read).await;
    });

    let connection_closed_write = connection_closed.clone();
    let write_task = tokio::spawn(async move {
        write_to_peer(writer, writer_source, connection_closed_write).await;
    });

    select! {
        _ = write_task => {
            read_task.abort();
        }
    }
}

pub async fn read_from_peer<R>(reader: R, connection_closed: Arc<Mutex<bool>>)
where
    R: AsyncRead + Unpin,
{
    let mut buffer = BufReader::new(reader);
    loop {
        let mut buff = [0; 1024];

        let size = match buffer.read(&mut buff).await {
            Ok(0) => {
                let connection_closed_msg = "Connection closed by the peer".red().bold();
                println!("{}", connection_closed_msg);
                let mut connection_closed = connection_closed.lock().await;

                *connection_closed = true;

                break;
            }
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to read from socket: {:?}", e);
                break;
            }
        };

        if let Ok(msg) = std::str::from_utf8(&buff[..size]) {
            let msg = msg.trim_end();
            let timestamp = get_timestamp();

            println!("{} {}: {}", timestamp.blue(), "Peer".green(), msg);
        } else {
            println!("Received non-UTF8 data");
        }
    }
}

pub async fn write_to_peer<W, S>(mut writer: W, mut source: S, connection_closed: Arc<Mutex<bool>>)
where
    W: AsyncWriteExt + Unpin,
    S: MessageSource + Send,
{
    while let Some(msg) = source.next_message().await {
        if &msg == "/exit\n" {
            println!("Exit the discussion");
            if !*connection_closed.lock().await {
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

        if !*connection_closed.lock().await {
            if let Err(e) = writer.write_all(msg.as_bytes()).await {
                // For keeping the user into the chat room without error
                // when the peer is disconnected
                if e.kind() != std::io::ErrorKind::BrokenPipe {
                    println!("Error while sending message: {}", e);
                    break;
                }
            }
        }
    }
}

#[async_trait]
pub trait MessageSource {
    async fn next_message(&mut self) -> Option<String>;
}

pub struct LockedReceiver {
    receiver: Arc<Mutex<Receiver<String>>>,
}

impl LockedReceiver {
    pub fn new(receiver: Arc<Mutex<Receiver<String>>>) -> Self {
        LockedReceiver { receiver }
    }
}

#[async_trait]
impl MessageSource for LockedReceiver {
    async fn next_message(&mut self) -> Option<String> {
        let mut receiver = self.receiver.lock().await;
        receiver.recv().await
    }
}

pub struct StdinMessageSource {
    reader: tokio::io::BufReader<tokio::io::Stdin>,
}

impl StdinMessageSource {
    pub fn new() -> Self {
        StdinMessageSource {
            reader: BufReader::new(tokio::io::stdin()),
        }
    }
}

#[async_trait]
impl MessageSource for StdinMessageSource {
    async fn next_message(&mut self) -> Option<String> {
        let mut input = String::new();
        match self.reader.read_line(&mut input).await {
            Ok(0) => None,
            Ok(_) => Some(input),
            Err(_) => None,
        }
    }
}
