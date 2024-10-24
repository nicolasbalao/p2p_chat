use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

pub async fn connect(addr: &str, port: &str) -> Result<(), std::io::Error> {
    let connection = tokio::net::TcpStream::connect(format!("{}:{}", addr, port)).await?;

    println!("Connected with: {}:{}", addr, port);

    let (reader, mut writer) = connection.into_split();

    let read_task = tokio::spawn(async move {
        let mut buffer = BufReader::new(reader);
        loop {
            let mut buff = [0; 1024];
            let n = match buffer.read(&mut buff).await {
                Ok(0) => {
                    // If we read 0 bytes, that means the connection is closed
                    println!("Connection closed by peer");
                    println!("/exit for end the discussion");
                    break;
                }
                Ok(n) => n, // `n` is the number of bytes read
                Err(e) => {
                    println!("Failed to read from socket: {:?}", e);
                    break;
                }
            };

            if let Ok(text) = std::str::from_utf8(&buff[..n]) {
                println!("Peer: {}", text);
            } else {
                println!("Received non-UTF8 data");
            }
        }
    });

    let write_task = tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut input = String::new();

        loop {
            input.clear();
            match reader.read_line(&mut input).await {
                Ok(0) => {
                    println!("Stdin closed");
                }
                Ok(_) => {
                    // Handle shutdown

                    if &input == "/exit\n" {
                        println!("Exit the discussion");
                        writer.shutdown().await.expect("Failed to shutdown writer");
                        break;
                    }

                    if let Err(e) = writer.write_all(input.as_bytes()).await {
                        println!("Failed to write to socker {}", e);
                        break;
                    }
                }
                Err(e) => {
                    println!("Failed to read from stdin {}", e);
                    break;
                }
            }
        }
    });

    // select! {
    //     _ = read_task => {},
    //     _ = write_task => {}
    // }

    tokio::try_join!(read_task, write_task)?;

    println!("Write and Read task is ended");

    Ok(())
}
