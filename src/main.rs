use std::{
    io::{stdout, Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() -> std::io::Result<()> {
    // Listen on port 3425
    let listener = TcpListener::bind("0.0.0.0:8989")?;
    println!("Server listening on 0.0.0.0:8989");

    // Read message
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) {
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
