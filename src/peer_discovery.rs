use core::str;
use std::{io, sync::Arc, thread, time::Duration};

use tokio::sync::Mutex;
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
        thread::sleep(Duration::from_millis(duration_millis));

        // Send back the response Uuid:Port
        sock.send_to(msg.as_bytes(), addr).await?;
    }
}

pub async fn send_hello_broadcast(app: Arc<Mutex<App>>) -> io::Result<()> {
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:54353").await?;

    socket.set_broadcast(true)?;
    println!("Broadcast addr: {:?}", socket.local_addr());

    let mut msg = String::new();

    {
        let app = app.lock().await;
        msg = format!("{}:{}", app.uuid, app.addr.port());
    }

    socket
        .send(msg.as_bytes())
        .await
        .expect("Failed to send broadcast request");

    println!("Awaiting response ...");
    let mut buf = [0; 1024];
    while let Ok((n, mut addr)) = socket.recv_from(&mut buf).await {
        let request = str::from_utf8(&buf[..n])
            .expect("Non valide UTF-8")
            .trim_end();

        let mut informations = request.split(":");

        let uuid = informations.next().expect("Failed to have uuid of peer");
        let port = informations.next().expect("Failed to have port of peer");

        let uuid = uuid.parse::<Uuid>().expect("Failed to parse the Uuid");
        addr.set_port(
            port.parse::<u16>()
                .expect("Failed to parse the port to u16"),
        );

        {
            let mut app = app.lock().await;
            app.peers.insert(uuid, addr);
            println!("Peers: {:?}", app.peers);
        }
    }

    Ok(())
}
