use std::io;
use tokio::io::AsyncBufReadExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::utils::load_user_path;


pub mod utils;
mod path_manager;
mod auth_flow;
mod menu;



// Entry point del client applicativo.

#[tokio::main]
async fn main() -> io::Result<()> {

    print!("\x1B[2J\x1B[1;1H");
    println!("Starting Client application...");

    loop {

        let mut stream = match TcpStream::connect("127.0.0.1:8080").await {

            Ok(s) => s,

            Err(e) => {
                eprintln!("\x1b[31mFailed to connect to server: {}\x1b[0m", e);
                return Ok(());
            }

        };

        println!("\x1b[32mConnected to server!\x1b[0m");

        let username = auth_flow::register_and_login(&mut stream).await;
        let user_dir = format!("client/users/{}", username);
        let file_path = format!("{}/route.txt", user_dir);

        path_manager::ensure_user_path_exists(&username, &user_dir, &file_path).await?;

        let route = load_user_path(&file_path, &username).await?;
        
        let last_position = route.last().map(|up| (up.coordinates.get_latitude(), up.coordinates.get_longitude()));

        if route.is_empty() {

            println!("\x1b[33mNo previous positions found, starting a fresh route.\x1b[0m");

        } else {

            println!("\x1b[36mLoaded {} previous positions from history.\x1b[0m", route.len());

        }

        println!("Initializing all communication sockets...");
    
        let (read_half, write_half) = stream.into_split();
        let reader = tokio::io::BufReader::new(read_half).lines();
        let reader_task = utils::spawn_reader_task(reader);

        let (tx, rx) = mpsc::channel::<utils::Outgoing>(32);
        let tx_positions = tx.clone();

        let writer_task = utils::spawn_writer_task(rx, write_half);
        let position_task = utils::spawn_position_task(file_path, tx_positions, username.clone(), last_position);
        
        println!("\x1b[32mCommunication channels initialized.\x1b[0m");

        let action = menu::run_main_menu(&tx, &username).await;

        match action {

            menu::MenuAction::Logout => {

                position_task.abort();
                reader_task.abort();

                drop(tx);

                let _ = writer_task.await;

                println!("Returning to main menu...\n");

                continue;
            }
        }
    }

}