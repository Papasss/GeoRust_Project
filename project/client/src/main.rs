use std::io;
use tokio::io::AsyncBufReadExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::Duration;

use shared::messages::Message;
use shared::utils::send_packet;
use utils::load_user_path;

use crate::utils::Outgoing;

pub mod utils;
mod path_manager;
mod auth_flow;
mod menu;


// Avvia l'applicazione client orchestrando l'autenticazione, la rete e l'interfaccia.
// Apre la connessione TCP, avvia i task paralleli per la lettura e scrittura dei socket,
// innesca l'invio temporizzato delle coordinate e lancia il menu utente interattivo.
#[tokio::main]
async fn main() -> io::Result<()> {
    println!("Starting Client application...");

    loop {
        let mut stream = match TcpStream::connect("127.0.0.1:8080").await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to connect to server: {}", e);
                return Ok(());
            }
        };
        println!("Connected to server!");

        let username = auth_flow::run_auth_flow(&mut stream).await;
        
        let user_dir = format!("client/{}", username);
        let file_path = format!("{}/route.txt", user_dir);

        path_manager::ensure_user_path_exists(&username, &user_dir, &file_path).await?;

        let route = load_user_path(&file_path, &username).await?;

        if route.is_empty() {
            println!("No positions found in {}", file_path);
            return Ok(());
        }
    
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = tokio::io::BufReader::new(read_half).lines();

        tokio::spawn(async move {
            while let Ok(Some(line)) = reader.next_line().await {
                if let Ok(msg) = serde_json::from_str::<Message>(&line) { 
                    match msg {
                        Message::IncomingDirectMessage { from, text } => {
                            println!("\n[Private message from {}]: {}", from, text);
                        }
                        Message::IncomingBroadcastMessage { from, text } => {
                            println!("\n[Broadcast from {}]: {}", from, text);
                        }
                        _ => {} 
                    }
                }
            }
        });

        let (tx, mut rx) = mpsc::channel::<Outgoing>(32);

        let writer_task = tokio::spawn(async move {
            while let Some(packet) = rx.recv().await {
                match packet {
                    Outgoing::Chat(msg) => { let _ = send_packet(&mut write_half, &msg).await; },
                    Outgoing::Position(upd) => { let _ = send_packet(&mut write_half, &upd).await; },
                }
            }
        });
                
        let username_clone = username.clone();
        let tx_positions = tx.clone();
        let position_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            for update in route {
                interval.tick().await;
                if tx_positions.send(Outgoing::Position(update)).await.is_err() {
                    break;
                }
            }
            println!("\nRoute completed for user {}.", username_clone);
        });
        
        let action = menu::run_main_menu(&tx, &username).await;

        match action {
            menu::MenuAction::Logout => {
                position_task.abort();
                drop(tx);
                let _ = writer_task.await;
                println!("Returning to main menu...\n");
                continue;
            }
        }
    }

}