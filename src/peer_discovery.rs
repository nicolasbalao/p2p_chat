use core::str;
use std::{io, net::SocketAddr, sync::Arc, time::Duration};

use crossterm::style::Stylize;
use tokio::{sync::Mutex, time::sleep};
use uuid::Uuid;

use crate::App;

pub async fn server_udp(app: Arc<Mutex<App>>) -> io::Result<()> {
    let sock = tokio::net::UdpSocket::bind("0.0.0.0:52345").await?;
    let mut buf = [0; 1024];

    loop {
        let (len, addr) = sock.recv_from(&mut buf).await?;

        let request = str::from_utf8(&buf[..len])
            .expect("Non valide UTF-8")
            .trim_end();

        let mut informations = request.split(":");

        let uuid = informations.next().expect("Failed to have uuid of peer");
        let port = informations.next().expect("Failed to have port of peer");

        let uuid = uuid.parse::<Uuid>().expect("Failed to parse the Uuid");
        let mut com_addr = addr.clone();
        com_addr.set_port(
            port.parse::<u16>()
                .expect("Failed to parse the port to u16"),
        );

        {
            let mut app = app.lock().await;

            if uuid != app.uuid {
                app.peers.insert(uuid, com_addr);

                let new_peer_msg = format!("Peers connected say hello at {}", com_addr).blue();

                println!("{}", new_peer_msg);
            }
        }

        // Add delay for send response to the broadcast request
        let duration_millis = rand::random::<u64>() % 2500;
        sleep(Duration::from_millis(duration_millis)).await;

        let broadcast_msg = {
            let app = app.lock().await;
            format!("{}:{}", app.uuid, app.addr.port())
        };
        // Send back the response Uuid:Port
        sock.send_to(broadcast_msg.as_bytes(), addr).await?;
    }
}

pub async fn send_hello_broadcast(app: Arc<Mutex<App>>) -> io::Result<()> {
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;

    socket.set_broadcast(true)?;

    let msg = {
        let app = app.lock().await;
        format!("{}:{}", app.uuid, app.addr.port())
    };

    let broadcast_socket = "255.255.255.255:52345"
        .parse::<SocketAddr>()
        .expect("Failed to parse to broadcast socket");

    socket
        .send_to(msg.as_bytes(), broadcast_socket)
        .await
        .expect("Failed to send broadcast request");

    let mut buf = [0; 1024];

    loop {
        tokio::select! {
            // Receive a response from any peer
            result = socket.recv_from(&mut buf) => {
                match result {
                    Ok((len, addr)) => {
                        let request = str::from_utf8(&buf[..len])
                            .expect("Non valide UTF-8")
                            .trim_end();

                        let mut informations = request.split(":");

                        let uuid = informations.next().expect("Failed to have uuid of peer");
                        let port = informations.next().expect("Failed to have port of peer");

                        let uuid = uuid.parse::<Uuid>().expect("Failed to parse the Uuid");
                        let mut com_addr = addr.clone();
                        com_addr.set_port(
                            port.parse::<u16>()
                                .expect("Failed to parse the port to u16"),
                        );

                        {
                            let mut app = app.lock().await;

                            if uuid != app.uuid {
                                app.peers.insert(uuid, com_addr);
                            }
                        }


                        },
                    Err(e) => {
                        eprintln!("Failed to receive response: {}", e);
                        break;
                    }
                }
            }
            // Timeout after waiting to avoid an infinite loop
            _ = sleep(Duration::from_secs(5)) => {
                break;
            }
        }
    }

    {
        let app = app.lock().await;

        if !app.peers.is_empty() {
            let msg = "Peer connected: ".yellow();
            println!("{}", msg);

            app.list_peers();
        }
    }

    Ok(())
}
