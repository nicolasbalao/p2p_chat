use core::{net, str};
use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use get_if_addrs::{get_if_addrs, IfAddr, Ifv4Addr, Ifv6Addr, Interface};
use tokio::{
    sync::{broadcast, Mutex},
    time::sleep,
};
use uuid::Uuid;

use crate::App;

pub async fn server_udp(app: Arc<Mutex<App>>) -> io::Result<()> {
    let sock = tokio::net::UdpSocket::bind("0.0.0.0:52345").await?;
    println!("Recon server lunched");
    let mut buf = [0; 1024];
    let mut msg = String::new();

    {
        let app = app.lock().await;
        msg = format!("{}:{}", app.uuid, app.addr.port());
    }

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
            app.peers.insert(uuid, com_addr);
            println!("Peers: {:?}", app.peers);
        }

        println!("Request : {:?}", request);

        // Add delay for send response to the broadcast request
        let duration_millis = rand::random::<u64>() % 2500;
        sleep(Duration::from_millis(duration_millis)).await;

        // Send back the response Uuid:Port
        sock.send_to(msg.as_bytes(), addr).await?;
    }
}

pub async fn send_hello_broadcast(app: Arc<Mutex<App>>) -> io::Result<()> {
    let my_ip = get_local_ip().unwrap();

    println!("Your interface network information: {:?}", my_ip.addr);

    let broadcast = match my_ip.addr {
        IfAddr::V4(v4_addr) => v4_addr,
        _ => {
            eprintln!("No ip 4");
            std::process::exit(1);
        }
    };
    println!("Broadcast: {:?}", broadcast.ip);

    let broadcast_ip = broadcast.ip;

    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;

    socket.set_broadcast(true)?;
    println!("My addr: {:?}", socket.local_addr());
    println!("Broadcast addr: {:?}", socket.broadcast());

    let mut msg = String::new();

    {
        let app = app.lock().await;
        msg = format!("{}:{}", app.uuid, app.addr.port());
    }

    let broadcast_socket = SocketAddr::new(IpAddr::V4(broadcast_ip), 5234);

    socket
        .send_to(msg.as_bytes(), broadcast_socket)
        .await
        .expect("Failed to send broadcast request");

    println!("Awaiting response ...");

    let mut buf = [0; 1024];

    loop {
        tokio::select! {
            // Receive a response from any peer
            result = socket.recv_from(&mut buf) => {
                match result {
                    Ok((len, addr)) => {
                        println!("Received response from {}: {:?}", addr, &buf[..len]);
                    },
                    Err(e) => {
                        eprintln!("Failed to receive response: {}", e);
                        break;
                    }
                }
            }
            // Timeout after waiting to avoid an infinite loop
            _ = sleep(Duration::from_secs(5)) => {
                println!("Listening for responses timed out.");
                break;
            }
        }
    }

    Ok(())
}

fn get_local_ip() -> Option<Interface> {
    for iface in get_if_addrs().unwrap() {
        if !iface.is_loopback() {
            return Some(iface);
        }
    }
    None
}
