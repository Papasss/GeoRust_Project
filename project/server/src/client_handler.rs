use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;

use shared::messages::Message;
use shared::update_position::UpdatePosition;
use shared::utils::send_packet;
use crate::server_state::ServerState;



// Rappresenta l'intero ciclo di vita di un client connesso al server.
// Separa il socket in lettura e scrittura per evitare blocchi e orchestra in sequenza
// autenticazione, sessione attiva e disconnessione sicura.
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



// Gestisce la prima fase di connessione filtrando le richieste non autorizzate.
// Attende input dal client per eseguire il login o la registrazione, bloccando 
// temporaneamente lo stato del server e rispondendo in base all'esito.
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



// Inizializza il canale asincrono per mantenere aperta la comunicazione in uscita.
// Avvia un task in background che attende passivamente i messaggi inseriti nel canale
// e li recapita al client inviandoli fisicamente sul socket TCP.
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



// Mantiene aperto il canale di ascolto smistando i comandi e le posizioni del client.
// Tenta un doppio parsing del JSON per far convivere la messaggistica e le coordinate,
// indirizzando correttamente i pacchetti verso le rispettive logiche del server.
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



// Rimuove la sessione dell'utente dallo stato e interrompe il flusso in uscita.
// Spegne definitivamente il task dedicato alla scrittura, liberando
// la memoria non appena l'utente chiude volontariamente la connessione.
async fn disconnect_client(username: &str, state: &Arc<Mutex<ServerState>>, writer_task: JoinHandle<()>) {
    let mut server_state_lock = state.lock().await;
    server_state_lock.logout(username);
    drop(server_state_lock);
    
    writer_task.abort();

    println!("\t\t\t\tUser disconnected: {username}");
}