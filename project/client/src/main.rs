use std::io;
use tokio::io::AsyncBufReadExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use utils::load_user_path;


pub mod utils;
mod path_manager;
mod auth_flow;
mod menu;



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

        let username = auth_flow::register_and_login(&mut stream).await;
        let user_dir = format!("client/users{}", username);
        let file_path = format!("{}/route.txt", user_dir);

        path_manager::ensure_user_path_exists(&username, &user_dir, &file_path).await?;

        let route = load_user_path(&file_path, &username).await?;

        if route.is_empty() {

            println!("No positions found in {}", file_path);
            return Ok(());

        }

        println!("Initializing all communication sockets");
    
        // Avvio task di lettura. Si occupa di ricevere unicamente messaggi
        // diretti e di broadcast
        
        let (read_half, write_half) = stream.into_split();
        let reader = tokio::io::BufReader::new(read_half).lines();
        let reader_task = utils::spawn_reader_task(reader);

        // Avvio task di scrittura e coordinate

        let (tx, rx) = mpsc::channel::<utils::Outgoing>(32);
        let tx_positions = tx.clone();

        let writer_task = utils::spawn_writer_task(rx, write_half);
        let position_task = utils::spawn_position_task(route, tx_positions, username.clone());
        
        println!("Communication channels initialized");

        // Avvia il menu interattivo

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