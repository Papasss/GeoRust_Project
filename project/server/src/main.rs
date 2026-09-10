use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use crate::server_state::ServerState;

mod server_state;
mod tracker_state;
mod auth;
mod analytics;
mod client_handler;

#[tokio::main]
async fn main() {

    println!("Starting CPU's usage");

    ServerState::start_log_cpu_usage().await;

    println!("Starting Control Server...");

    let mut initial_state = ServerState::new();
    
    initial_state.load_accounts().await;
    
    let state = Arc::new(Mutex::new(initial_state));

    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to port");
        
    println!("Server listening on 127.0.0.1:8080");

    loop {

        let (socket, addr) = listener.accept().await.unwrap(); 
        
        println!("New client connected: {addr}");

        let state_clone = Arc::clone(&state); 

        tokio::spawn(async move { 
            client_handler::handle_client(socket, state_clone).await;
        });
    }
}
