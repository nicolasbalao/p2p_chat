use std::{io::Error, sync::Arc};

use tokio::sync::mpsc::Receiver;

use crate::{
    ui::{clear_screen, print_welcome_message, start_chat_screen},
    utils::{handle_chat, LockedReceiver},
};

pub async fn start(port: &str, rx: Receiver<String>) -> Result<(), Error> {
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    let mut is_first_connection = true;

    loop {
        if !is_first_connection {
            clear_screen();
            print_welcome_message(port);
        }
        let (handle, addr) = listener.accept().await?;

        start_chat_screen(&addr.to_string()).await;

        let (reader, writer) = handle.into_split();

        let rx_clone = Arc::clone(&rx);
        let locked_rx = LockedReceiver::new(rx_clone);
        handle_chat(reader, writer, locked_rx).await;

        is_first_connection = false;
    }
}
