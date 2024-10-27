use std::{io::Error, sync::Arc};

use tokio::{
    select,
    sync::{mpsc::Receiver, Mutex},
};

use crate::{
    ui::{clear_screen, print_welcome_message, start_chat_screen},
    utils::{read_from_peer, write_to_peer, LockedReceiver},
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

        let connection_closed = Arc::new(Mutex::new(false));

        start_chat_screen(&addr.to_string()).await;

        let (reader, writer) = handle.into_split();

        let connection_closed_read = connection_closed.clone();
        let read_task = tokio::spawn(async move {
            read_from_peer(reader, connection_closed_read).await;
        });

        let rx_clone = Arc::clone(&rx);

        let connection_closed_write = connection_closed.clone();
        let write_task = tokio::spawn(async move {
            let locked_rx = LockedReceiver::new(rx_clone);
            write_to_peer(writer, locked_rx, connection_closed_write).await
        });

        select! {
            _ = write_task => {
                read_task.abort();
            }
        }
        println!("End discussion");
        is_first_connection = false;
    }
}
