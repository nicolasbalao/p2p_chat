use core::str;
use std::{
    collections::HashMap,
    env::{self, Args},
    io::stdin,
    net::{SocketAddr, SocketAddrV4},
    sync::Arc,
};

use crossterm::style::Stylize;
use peer_discovery::send_hello_broadcast;
use rand::seq::SliceRandom;
use sha2::{Digest, Sha256};
use tokio::sync::{mpsc, Mutex};
use ui::{clear_screen, print_welcome_message};
use uuid::Uuid;

mod client;
mod peer_discovery;
mod server;
mod ui;
mod utils;

#[derive(Debug, Clone)]
struct Peer {
    addr: SocketAddr,
    name: String,
}

#[derive(Debug)]
struct App {
    addr: SocketAddr,
    uuid: Uuid,
    peers: HashMap<Uuid, Peer>,
    name: String,
    token: Vec<u8>,
}

impl App {
    pub fn new(addr: SocketAddr, name: String) -> Self {
        // TODO: Make it secure
        let shared_key = "p2p_app";
        let mut hasher = Sha256::new();
        hasher.update(shared_key);
        let token = hasher.finalize().to_ascii_lowercase();
        App {
            addr,
            uuid: Uuid::new_v4(),
            peers: HashMap::new(),
            name,
            token,
        }
    }

    pub fn add_peer(&mut self, uuid: Uuid, peer: Peer) {
        self.peers.insert(uuid, peer);
    }

    pub fn list_peers(&self) {
        let line_length = 40;
        let title = " PEER LIST ";

        // Print the top border with the title centered
        let padding = (line_length - title.len()) / 2;
        println!(
            "{}{}{}",
            "-".repeat(padding),
            title,
            "-".repeat(line_length - padding - title.len())
        );

        // Print each peer with styled formatting
        for (_, peer) in self.peers.clone().into_iter() {
            println!("| {:^10} | {:^23} |", peer.name, peer.addr);
        }

        // Print the bottom border
        println!("{}", "-".repeat(line_length));
    }
}

#[tokio::main]
async fn main() {
    let args = env::args();
    if let Err(e) = run(args).await {
        let msg = format!("Error: {}", e).red();
        eprintln!("{}", msg);
        std::process::exit(1);
    }
}

async fn run(mut args: Args) -> std::io::Result<()> {
    let port = args.nth(1).expect("Failed to read port arg");
    let name = args.next().unwrap_or(default_name());
    let port_clone = port.clone();

    let (tx, rx) = mpsc::channel(100);

    let addr = format!("127.0.0.1:{}", port).parse::<SocketAddr>().unwrap();

    let app = App::new(addr, name);
    let app_name = app.name.clone();

    let app_clone = Arc::new(Mutex::new(app));

    // Broadcast
    let app_hello = app_clone.clone();
    tokio::spawn(async move {
        if let Err(e) = send_hello_broadcast(app_hello).await {
            eprintln!("Error broadcast: {}", e);
            std::process::exit(1);
        };
    });

    // Communication server
    let app_server = app_clone.clone();
    tokio::spawn(async move {
        if let Err(e) = server::start(&port, rx, app_server).await {
            eprintln!("Server error: {e}");
        }
    });

    // Discovery server
    let app_dicovery = app_clone.clone();
    tokio::spawn(async move {
        if let Err(e) = peer_discovery::server_udp(app_dicovery).await {
            eprintln!("Error discovery server: {}", e);
        }
    });

    let mut input = String::new();
    let stdin = stdin();

    // Welcome message

    print_welcome_message(&port_clone, &app_name);

    loop {
        input.clear();
        stdin
            .read_line(&mut input)
            .expect("Failed to read stdin input");

        if input.starts_with("/") {
            let (command, args) = prepare_input(&input);
            match command {
                "/connect" => {
                    let args = match args {
                        Some(args) => args,
                        None => {
                            let msg = "Invalide syntax => /connect IP:PORT".red();
                            eprintln!("{}", msg);
                            continue;
                        }
                    };

                    let peer_socker_addr = match args.parse::<SocketAddrV4>() {
                        Ok(s) => s,
                        Err(e) => {
                            let error_msg = format!("Error: {}", e).red();
                            eprintln!("{}", error_msg);
                            continue;
                        }
                    };

                    if let Err(e) = client::connect(peer_socker_addr).await {
                        eprintln!("Connection failed: {e}");
                    };

                    clear_screen();
                    print_welcome_message(&port_clone, &app_name);
                }
                "/peers" => {
                    let app = app_clone.clone();
                    let app = app.lock().await;
                    app.list_peers();
                }
                // REF this
                "/exit" => tx
                    .send(input.clone())
                    .await
                    .expect("Failed to send message in stdin channel"),
                _ => {
                    let unknown_command = format!("Command {} is unknown", command).red();
                    println!("{}", unknown_command);
                }
            }
        } else {
            tx.send(input.clone())
                .await
                .expect("Failed to send message in channel");
        }
    }
}

fn prepare_input(input: &str) -> (&str, Option<&str>) {
    let input_trimed = input.trim_end();
    let mut input_splited = input_trimed.split_whitespace();
    let command = input_splited.next().expect("Command not found");
    let args = input_splited.next();

    (command, args)
}
fn default_name() -> String {
    let funny_names = [
        "Captain Crunch",
        "Sir Laughs-a-Lot",
        "Pixel Pirate",
        "Chatty McChatterson",
        "GigaChad",
        "Meme Dream",
        "Emoji Overlord",
        "Data Whisperer",
        "Hacker Man",
        "LolzMaster",
        "Mr. Giggles",
        "Techie Wookie",
        "The Mighty Ping",
        "Lord of Bytes",
        "Ninja Byte",
        "404_NotFound",
        "Banter Wizard",
        "The Ping King",
        "Chat Crusader",
        "Cyber Sprite",
    ];

    funny_names
        .choose(&mut rand::thread_rng())
        .unwrap()
        .to_string()
}
