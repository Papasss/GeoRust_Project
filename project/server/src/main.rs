use std::sync::Arc;
use tokio::net::{TcpListener};
use tokio::sync::{Mutex};
use crate::server_state::ServerState;

mod server_state;
mod auth;
mod client_handler;

#[tokio::main]
async fn main() {

    println!("Starting Control Server...");
    println!("\tStarting loggin CPU usage...");

    ServerState::start_log_cpu_usage().await;

    println!("\t\tstarted!");

    // Un solo ServerState accessibile da tutti i task client tramite Arc

    let mut server_state= ServerState::new();
    
    println!("\tLoading accounts...");

    server_state.load_accounts("shared/data/accounts.json");

    println!("\t\tdone!");

    let state_arc = Arc::new(Mutex::new(server_state));

    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to port");

    println!("\tServer listening on 127.0.0.1:8080...");
    println!("\t\tStarting server loop...");

    loop {

        let (socket, addr) = listener.accept().await.unwrap();

        println!("\t\t\tNew client connected: {addr}");

        let state_arc_clone = state_arc.clone();

        tokio::spawn(async move {
            client_handler::handle_client(socket, state_arc_clone).await;
        });
    }

    // Per chi fa la gestione del server, possibilità in implementare un
    // comando di spegnimento? Altrimenti queste due stampe sono 
    // irraggiungibili e continueranno a dare warning.

    println!("\t\tended loop.");
    println!("\tended control server.");
}