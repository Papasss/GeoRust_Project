use std::sync::Arc;
use shared::coordinates::Coordinates;
use shared::send_packet;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use shared::messages::{AnalyticsField, AnalyticsPeriodMessage, Message};
use shared::update_position::UpdatePosition;

use crate::server_state::ServerState;
use crate::analytics::{
    analyze_movement,
    AnalysisPeriod,
    AnalyticsConfig,
    MovementStatistics,
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
                        if let Err(e) = server_state_lock.save_accounts().await {
                            eprintln!("Errore nel salvataggio degli account: {e}");
                        }
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

                Message::AnalyticsRequest { field, period } => {

                    let response = handle_analytics_request(username, field, period, state).await;

                    send_to_logged_user(username, response, state).await;
                }

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



// Invia un messaggio al client autenticato usando il canale salvato in ServerState

async fn send_to_logged_user( username: &str, msg: Message, state: &Arc<Mutex<ServerState>>,) {

    let tx = {
        let server_state_lock = state.lock().await;
        server_state_lock.connections.get(username).cloned()
    };
        

    if let Some(tx) = tx {
        let _ = tx.send(msg).await;
    }
}



// Converte il periodo ricevuto dal client nel tipo usato dal modulo analytics.

fn convert_period(period: AnalyticsPeriodMessage) -> AnalysisPeriod {

    match period {

        AnalyticsPeriodMessage::CurrentDay => AnalysisPeriod::CurrentDay,

        AnalyticsPeriodMessage::CurrentWeek => AnalysisPeriod::CurrentWeek,

        AnalyticsPeriodMessage::CurrentMonth => AnalysisPeriod::CurrentMonth,

        AnalyticsPeriodMessage::Custom {

            start_timestamp,
            end_timestamp,

        } => AnalysisPeriod::Custom {

            start_timestamp,
            end_timestamp,

        },
    }
}



// Trasforma le statistiche calcolate in una risposta testuale leggibile dal client.

fn format_analytics_response(field: AnalyticsField, stats: &MovementStatistics) -> String {

    match field {

        AnalyticsField::Path => {
            let path = stats
                .path
                .iter()
                .map(|sample| format!("({:.5}, {:.5})", sample.get_latitude(), sample.get_longitude()))
                .collect::<Vec<String>>()
                .join(" -> ");

            format!("Tragitto percorso:\n{}", path)
        }

        AnalyticsField::TotalDistance => {
            format!("Distanza totale percorsa: {:.3} km", stats.total_distance_km)
        }

        AnalyticsField::AverageSpeed => {
            format!("Velocità media: {:.3} km/h", stats.average_speed_kmh)
        }

        AnalyticsField::MovementDuration => {
            format!(
                "Durata complessiva del movimento: {} secondi",
                stats.movement_duration.as_secs()
            )
        }

        AnalyticsField::PauseDuration => {
            format!(
                "Durata complessiva delle pause: {} secondi",
                stats.pause_duration.as_secs()
            )
        }

        AnalyticsField::All => {
            let path = stats
                .path
                .iter()
                .map(|sample| format!("({:.5}, {:.5})", sample.get_latitude(), sample.get_longitude()))
                .collect::<Vec<String>>()
                .join(" -> ");

            format!(
                "Tragitto percorso:\n{}\n\nDistanza totale: {:.3} km\nVelocità media: {:.3} km/h\nDurata movimento: {} secondi\nDurata pause: {} secondi",
                path,
                stats.total_distance_km,
                stats.average_speed_kmh,
                stats.movement_duration.as_secs(),
                stats.pause_duration.as_secs()
            )
        }
    }
}



// Gestisce la richiesta di analytics ricevuta dal client

async fn handle_analytics_request(username: &str,field: AnalyticsField,period: AnalyticsPeriodMessage,state: &Arc<Mutex<ServerState>>) -> Message {
    
    let history = {

        let server_state_lock = state.lock().await;

        match server_state_lock.users.get(username) {

            Some(tracker) => tracker.get_history().clone(),

            None => {

                return Message::AnalyticsErr("Nessuna posizione disponibile per questo utente.".to_string());
            
            }
        }
    };

    if history.is_empty() {

        return Message::AnalyticsErr("Cronologia posizioni vuota per questo utente.".to_string());

    }


    let samples: Vec<Coordinates> = history
        .iter()
        .map(|update| {update.coordinates.clone()})
        .collect();

    let analysis_period = convert_period(period);
    let stats = analyze_movement(&samples, analysis_period, AnalyticsConfig::default());
    let response = format_analytics_response(field, &stats);

    Message::AnalyticsResponse(response)
}



// Rimuove la sessione dell'utente dallo stato e chiude le connessioni.

async fn disconnect_client(username: &str, state: &Arc<Mutex<ServerState>>, writer_task: JoinHandle<()>) {
    
    let mut server_state_lock = state.lock().await;
    
    server_state_lock.logout(username);
    drop(server_state_lock);
    
    writer_task.abort();

    println!("\t\t\t\tUser disconnected: {username}");
}