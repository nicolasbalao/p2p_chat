use std::io::{stdin, Error};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    select,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Read args client / server

    let args = std::env::args().collect::<Vec<String>>();

    let mode = &args[1];

    match mode.as_str() {
        "client" => {
            let mut input = String::new();
            let stdin = stdin();
            println!("IP:PORT: ");
            stdin
                .read_line(&mut input)
                .expect("Failed to read stdin input");

            let input = input.trim_end();
            if !input.is_empty() {
                let input_splited: Vec<&str> = input.split(":").collect();

                let addr = input_splited[0];
                let port = input_splited[1];

                client(addr, port).await?
            } else {
                client("0.0.0.0", "8989").await?;
            }
        }
        "server" => {
            server().await?;
        }
        _ => {
            println!("Invalid mode");
        }
    }

    Ok(())
}

async fn client(addr: &str, port: &str) -> Result<(), std::io::Error> {
    let connection = tokio::net::TcpStream::connect(format!("{}:{}", addr, port)).await?;

    println!("Connected with: {}:{}", addr, port);

    let (reader, writer) = connection.into_split();
    read_write(reader, writer).await;

    println!("Connection closed by the server");

    Ok(())
}

async fn server() -> Result<(), Error> {
    // Listen on port 3425
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8989").await?;
    println!("Server listening on 0.0.0.0:8989");

    let (handle, addr) = listener.accept().await?;

    println!("New client: {addr}");

    let (reader, writer) = handle.into_split();

    read_write(reader, writer).await;
    println!("Connection closed by the client");

    Ok(())
}

// Utils

async fn read_write<R, W>(mut reader: R, mut writer: W)
where
    R: AsyncRead + Unpin + Sized + Send + 'static,
    W: AsyncWrite + Unpin + Sized + Send + 'static,
{
    let client_read = tokio::spawn(async move {
        let _ = tokio::io::copy(&mut reader, &mut tokio::io::stdout()).await;
    });

    let client_write =
        tokio::spawn(async move { tokio::io::copy(&mut tokio::io::stdin(), &mut writer).await });

    select! {
        _ =  client_read => {},
        _ = client_write => {}
    }
}
