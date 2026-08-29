use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use shared::messages::{AnalyticsField, AnalyticsPeriodMessage, Message};
// Il client invia le posizioni come UpdatePosition, non come Message.
// Per questo il server deve riuscire a deserializzare anche questo tipo.
use shared::update_position::UpdatePosition;
use crate::server_state::ServerState;
// Modulo analytics: contiene la funzione sviluppata per calcolare
// tragitto, distanza, velocità media, durata movimento e pause.
use crate::analytics::{
    analyze_movement,
    AnalysisPeriod,
    AnalyticsConfig,
    MovementStatistics,
    PositionSample,
};



// Rappresenta l'intero ciclo di vita di un client connesso al server.

pub async fn handle_client(socket: TcpStream, state: Arc<Mutex<ServerState>>) {
    
    let (read_half, mut write_half) = socket.into_split();
    let mut reader = BufReader::new(read_half).lines();

    let username = match authenticate_client(&mut reader, &mut write_half, &state).await {
        Some(name) => name,
        None => return,
    };

    println!("\t\t\t\tUser authenticated successfully: {username}");

    let writer_task = setup_active_session(&username, write_half, &state).await;

    process_client_messages(&mut reader, &state, &username).await;

    disconnect_client(&username, &state, writer_task).await;
}



// Registrazione e/o login del client.

async fn authenticate_client(reader: &mut Lines<BufReader<OwnedReadHalf>>, write_half: &mut OwnedWriteHalf, state: &Arc<Mutex<ServerState>>) -> Option<String> {
    
    loop {

        let line = match reader.next_line().await {

            Ok(Some(line)) => line,

            _ => {
                println!("\t\t\t\tClient disconnected during authentication phase");
                return None;
            }
        };

        let msg: Message = match serde_json::from_str(&line) {

            Ok(m) => m,
            Err(_) => continue,

        };

        match msg {

            Message::Register { username, password } => {

                let mut server_state_lock = state.lock().await;
                let response = match server_state_lock.register(&username, &password) {

                    Ok(()) => {
                        let _ = server_state_lock.save_accounts("../shared/data/accounts.json").await;
                        Message::RegisterOk
                    }

                    Err(e) => Message::RegisterErr(e.to_string()),
                };

                drop(server_state_lock);
                
                let _ = send_packet(&mut *write_half, &response).await;
            }

            Message::Login { username, password } => {

                let server_state_lock = state.lock().await;

                if server_state_lock.is_online(&username) {

                    drop(server_state_lock);

                    let _ = send_packet(&mut *write_half, &Message::LoginErr(
                        format!("Session already active for '{}'", username)
                    )).await;
                    continue;

                }

                let auth_result = server_state_lock.authenticate(&username, &password);
                
                drop(server_state_lock);

                match auth_result {

                    Ok(()) => {
                        let _ = send_packet(&mut *write_half, &Message::LoginOk).await;
                        return Some(username);
                    }

                    Err(e) => {
                        let _ = send_packet(&mut *write_half, &Message::LoginErr(e.to_string())).await;
                    }
                }
            }

            _ => {
                let _ = send_packet(&mut *write_half, &Message::LoginErr(
                    "You must log in first".to_string()
                )).await;
            }
        }
    }
}



// Inizializza il canale per mantenere aperta la comunicazione in uscita.
// Avvia un task in background che attende i messaggi.

async fn setup_active_session(username: &str, mut write_half: OwnedWriteHalf, state: &Arc<Mutex<ServerState>>) -> JoinHandle<()> {
    
    let (tx, mut rx) = mpsc::channel::<Message>(32);

    {
        let mut server_state_lock = state.lock().await;
        server_state_lock.login(username, tx);
    }

    tokio::spawn(async move {

        while let Some(msg) = rx.recv().await {
            let _ = send_packet(&mut write_half, &msg).await;
        }

    })
}



// Smistando i comandi del client.

async fn process_client_messages(reader: &mut Lines<BufReader<OwnedReadHalf>>, state: &Arc<Mutex<ServerState>>, username: &str) {

    while let Ok(Some(line)) = reader.next_line().await {
        
        if let Ok(msg) = serde_json::from_str::<Message>(&line) {

            match msg {

                Message::SendDirectMessage { to, text } => {

                    let server_state_lock = state.lock().await;
                    let _ = server_state_lock.direct_message(
                        &to, 
                        Message::IncomingDirectMessage { from: username.to_string(), text }
                    ).await;
                }

                Message::SendBroadcastMessage { text } => {

                    let server_state_lock = state.lock().await;

                    server_state_lock.broadcast(
                        Message::IncomingBroadcastMessage { from: username.to_string(), text }
                    ).await;
                }

                _ => {}
            }
            continue;
        }

        if let Ok(update_position) = serde_json::from_str::<UpdatePosition>(&line) {

            let mut server_state_lock = state.lock().await;

            server_state_lock.process_packet(update_position);
            continue;

        }

        eprintln!("Unknown packet received from {}: {}", username, line);
    }
}



// Rimuove la sessione dell'utente dallo stato e chiude le connessioni.

async fn disconnect_client(username: &str, state: &Arc<Mutex<ServerState>>, writer_task: JoinHandle<()>) {
    
    let mut server_state_lock = state.lock().await;
    
    server_state_lock.logout(username);
    drop(server_state_lock);
    
    writer_task.abort();

    println!("\t\t\t\tUser disconnected: {username}");
}