use std::{
    io::{stdin, stdout, Read, Stdin, Stdout, Write},
    net::{TcpListener, TcpStream},
    sync::{atomic::AtomicBool, Arc},
    thread,
};

fn main() -> std::io::Result<()> {
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
            server()?;
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

fn server() -> std::io::Result<()> {
    // Listen on port 3425
    let listener = TcpListener::bind("0.0.0.0:8989")?;
    println!("Server listening on 0.0.0.0:8989");

    match listener.accept() {
        Ok((_socket, addr)) => {
            println!("New client : {addr}");

            let stdin = stdin();
            let stdout = stdout();
            // Read message
            // handle_client_message(_socket);
            let stream_clone = _socket.try_clone().expect("Failed to clone the stream");

            let keep_alive = Arc::new(AtomicBool::new(true));
            let keep_alive_clone = Arc::clone(&keep_alive);

            thread::spawn(move || {
                while keep_alive_clone.load(std::sync::atomic::Ordering::SeqCst) {
                    if !read_message_from_stream(&stream_clone, &stdout) {
                        keep_alive_clone.store(false, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            });

            let keep_alive_clone = Arc::clone(&keep_alive);
            thread::spawn(move || {
                while keep_alive_clone.load(std::sync::atomic::Ordering::SeqCst) {
                    send_message_from_stdin(&_socket, &stdin);
                }
            });

            while keep_alive.load(std::sync::atomic::Ordering::SeqCst) {}

            println!("Connection closed.");
            //     _socket
            //         .shutdown(std::net::Shutdown::Both)
            //         .expect("Failed to shutdown ");
        }
        Err(e) => println!("Couldn't get client: {e}"),
    }

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

    // let size = stream.read(&mut data).expect("Failed to read the stream ");

    // stdout
    //     .write(&data[0..size])
    //     .expect("Failed to write into stdout");
}
