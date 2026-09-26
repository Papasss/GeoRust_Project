use std::sync::Arc;
use tokio::net::TcpListener;
use log::{info, error, LevelFilter};
use chrono::Local;
use crate::server_state::ServerState;
use std::io::Write;


mod server_state;
mod tracker_state;
mod auth;
mod analytics;
mod client_handler;



// Inizializza l'applicazione configurando il server TCP

#[tokio::main]
async fn main() {

    env_logger::Builder::new()
        .format(|buf, record| {

            let style = buf.default_level_style(record.level());
            
            writeln!(
                buf,
                "{} [{style}{}{style:#}] [server] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    info!("[MONITOR]\tStarting system (CPU) monitoring...");

    ServerState::start_log_cpu_usage().await;

    info!("[SERVER]\tInitializing control server...");

    let initial_state = ServerState::new();

    initial_state.load_accounts().await;

    let state = Arc::new(initial_state);
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("[ERROR]\t\tUnable to start server on specified port");
        
    info!("[SERVER]\tListening on 127.0.0.1:8080");

    loop {
        
        let (socket, addr) = match listener.accept().await {
            
            Ok(pair) => pair,
            
            Err(e) => {
                error!("[ERROR]\t\tIncoming connection error: {e}");
                continue;
            }
        };
        
        info!("[NETWORK]\tNew physical connection established from: {addr}");

        let state_clone = Arc::clone(&state); 

        tokio::spawn(async move { 
            client_handler::handle_client(socket, state_clone).await;
        });
    }
}