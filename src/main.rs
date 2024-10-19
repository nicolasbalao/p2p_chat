use std::{
    io::{stdin, stdout, Error, Read, Stdin, Stdout, Write},
    net::TcpStream,
    thread,
};

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

                client(addr, port)
            } else {
                client("0.0.0.0", "8989");
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

fn client(addr: &str, port: &str) {
    let mut stream = TcpStream::connect(format!("{}:{}", addr, port)).expect("Failed to connect");

    println!("Connected with: {}:{}", addr, port);

    let stdin = stdin();
    let stdout = stdout();

    let stream_clone = stream.try_clone().expect("Failed to clone the stream");

    thread::spawn(move || loop {
        read_message_from_stream(&stream_clone, &stdout);
    });

    loop {
        send_message_from_stdin(&mut stream, &stdin);
    }
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

fn send_message_from_stdin(mut stream: &TcpStream, stdin: &Stdin) {
    let mut msg = String::new();

    stdin
        .read_line(&mut msg)
        .expect("Failed to read from the stdin");

    stream
        .write(msg.as_bytes())
        .expect("Failed to write message to the streame");
}

fn read_message_from_stream(mut stream: &TcpStream, mut stdout: &Stdout) -> bool {
    let mut data = [0 as u8; 50];

    match stream.read(&mut data) {
        Ok(size) if size > 0 => {
            stdout
                .write(&data[0..size])
                .expect("Failed to write to the stdout ");

            true
        }
        Ok(_) => {
            println!("Connection closed by the peer!");
            false
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            println!("Client disconnected unexpectedly");
            false
        }
        Err(e) => {
            println!("An error occured: {e}");
            true
        }
    }
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
