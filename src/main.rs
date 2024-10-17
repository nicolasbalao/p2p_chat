use std::{
    io::{stdin, stdout, Read, Write},
    net::{TcpListener, TcpStream},
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

            let input_splited: Vec<&str> = input.split(":").collect();

            let addr = input_splited[0];
            let port = input_splited[1];

            client(addr, port);
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

    loop {
        let mut message = String::new();

        print!("Message: ");
        stdin
            .read_line(&mut message)
            .expect("Failed to read from stdin");

        stream
            .write(message.as_bytes())
            .expect("Failed to write message to the streame");
    }
}

fn server() -> std::io::Result<()> {
    // Listen on port 3425
    let listener = TcpListener::bind("0.0.0.0:8989")?;
    println!("Server listening on 0.0.0.0:8989");

    // Read message
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client_message(stream);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_client_message(mut stream: TcpStream) {
    let mut data = [0 as u8; 50];

    let mut stdout = stdout();

    while match stream.read(&mut data) {
        Ok(size) => {
            stdout
                .write(&data[0..size])
                .expect("Failed to write stdout");
            true
        }
        Err(_) => {
            println!(
                "An error occured, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            false
        }
    } {}
}
