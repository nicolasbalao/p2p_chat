use core::str;
use std::{
    io,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use tokio::{sync::Mutex, time::sleep};
use uuid::Uuid;

use crate::{App, Peer};

#[derive(Serialize, Deserialize)]
struct DiscoveryMessage {
    uuid: Uuid,
    port: u16,
    name: String,
    timestamp: u64,
}

impl DiscoveryMessage {
    fn new(uuid: Uuid, port: u16, name: String) -> Self {
        DiscoveryMessage {
            uuid,
            port,
            name,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

pub async fn server_udp(app: Arc<Mutex<App>>) -> io::Result<()> {
    let upd_socket = tokio::net::UdpSocket::bind("0.0.0.0:52345").await?;
    let mut buf = [0; 1024];

    loop {
        let (len, recv_addr) = upd_socket.recv_from(&mut buf).await?;

        let request = str::from_utf8(&buf[..len])
            .expect("Non valide UTF-8")
            .trim_end();

        match from_str::<DiscoveryMessage>(request) {
            Ok(message) => {
                let mut peer_addr = recv_addr;
                peer_addr.set_port(message.port);

                {
                    let mut app = app.lock().await;

                    if message.uuid != app.uuid {
                        let peer = Peer {
                            name: message.name,
                            addr: peer_addr,
                        };
                        app.add_peer(message.uuid, peer);

                        let new_peer_msg =
                            format!("Peers connected say hello at {}", peer_addr).blue();

                        println!("{}", new_peer_msg);

                        // Send back discovery message
                        let duration_millis = rand::random::<u64>() % 2500;
                        sleep(Duration::from_millis(duration_millis)).await;

                        let discovery_message =
                            DiscoveryMessage::new(app.uuid, app.addr.port(), app.name.clone());

                        match to_string(&discovery_message) {
                            Ok(response) => {
                                upd_socket.send_to(response.as_bytes(), recv_addr).await?;
                            }
                            Err(e) => {
                                let msg = format!("Failed to convert to string: '{}'", e).red();
                                eprint!("{}", msg);
                                continue;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to serialze discovery message: '{}'", e);
            }
        };
    }
}

pub async fn send_hello_broadcast(app: Arc<Mutex<App>>) -> io::Result<()> {
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;

    socket.set_broadcast(true)?;

    let discovery_message = {
        let app = app.lock().await;
        DiscoveryMessage {
            uuid: app.uuid,
            port: app.addr.port(),
            name: app.name.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    };

    let broadcast_socket = "255.255.255.255:52345"
        .parse::<SocketAddr>()
        .expect("Failed to parse to broadcast socket");

    match to_string(&discovery_message) {
        Ok(message) => {
            socket
                .send_to(message.as_bytes(), broadcast_socket)
                .await
                .expect("Failed to send broadcast request");
        }
        Err(e) => {
            eprintln!("Failed to convert message to json: {}", e);
            return Ok(());
        }
    }

    let mut buf = [0; 1024];

    loop {
        tokio::select! {
            // Receive a response from any peer
            result = socket.recv_from(&mut buf) => {
                match result {
                    Ok((len, peer_discovery_addr)) => {
                        let request = str::from_utf8(&buf[..len])
                            .expect("Non valide UTF-8")
                            .trim_end();


                        match from_str::<DiscoveryMessage>(request) {
                            Ok(message) => {

                            let mut peer_addr = peer_discovery_addr;
                            peer_addr.set_port(
                                message.port
                            );

                            {
                                let mut app = app.lock().await;

                                if message.uuid != app.uuid {
                                    let peer = Peer{
                                        addr: peer_addr,
                                        name: message.name
                                    };


                                    app.add_peer(message.uuid, peer);
                                }
                            }

                            }
                            Err(e) => {
                                eprintln!("Failed to serialize message: '{}' ", e);
                                continue;
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
        } else {
            let msg = "No peers connected YET".yellow();
            println!("{}", msg);
        }
    }

    Ok(())
}
