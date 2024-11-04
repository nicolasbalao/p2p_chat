use core::str;
use std::{io, net::SocketAddr, sync::Arc, time::Duration};

use crossterm::style::Stylize;
use tokio::{sync::Mutex, time::sleep};
use uuid::Uuid;

use crate::App;

pub async fn server_udp(app: Arc<Mutex<App>>) -> io::Result<()> {
    let upd_socket = tokio::net::UdpSocket::bind("0.0.0.0:52345").await?;
    let mut buf = [0; 1024];

    loop {
        let (len, recv_addr) = upd_socket.recv_from(&mut buf).await?;

        let request = str::from_utf8(&buf[..len])
            .expect("Non valide UTF-8")
            .trim_end();

        if let Ok((uuid, port)) = extract_peers_information(request) {
            let mut peer_addr = recv_addr;
            peer_addr.set_port(port);

            {
                let mut app = app.lock().await;

                if uuid != app.uuid {
                    app.add_peer(uuid, recv_addr);

                    let new_peer_msg = format!("Peers connected say hello at {}", peer_addr).blue();

                    println!("{}", new_peer_msg);
                }
            }
        }

        // Add delay for send response to the broadcast request
        let duration_millis = rand::random::<u64>() % 2500;
        sleep(Duration::from_millis(duration_millis)).await;

        let response_message = {
            let app = app.lock().await;
            format!("{}:{}", app.uuid, app.addr.port())
        };
        // Send back the response Uuid:Port
        upd_socket
            .send_to(response_message.as_bytes(), recv_addr)
            .await?;
    }
}

pub async fn send_hello_broadcast(app: Arc<Mutex<App>>) -> io::Result<()> {
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;

    socket.set_broadcast(true)?;

    // Send broadcasting message
    // Message: UUID:PORT
    // Is the receiver who handle IP
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

                        if let Ok((uuid, port)) = extract_peers_information(request){

                            let mut com_addr = addr;
                            com_addr.set_port(
                                port
                            );

                            {
                                let mut app = app.lock().await;

                                if uuid != app.uuid {
                                    app.add_peer(uuid, addr);
                                }
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

fn extract_peers_information(request: &str) -> Result<(Uuid, u16), String> {
    if !request.contains(":") {
        return Err("Request message has bad format".to_string());
    }

    let mut informations = request.split(":");

    let uuid = match informations.next() {
        Some(uuid) => Uuid::parse_str(uuid).unwrap(),
        None => {
            return Err("No uuid found".to_string())
                .map_err(|e| format!("Failed to parse Uuid: {}", e))?;
        }
    };

    let port = match informations.next() {
        Some(port) => port
            .parse::<u16>()
            .map_err(|e| format!("Invalid port number '{}'. It mus be a number", e))?,
        None => {
            return Err("No Port found".to_string());
        }
    };

    Ok((uuid, port))
}
