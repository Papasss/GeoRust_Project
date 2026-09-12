use std::sync::Arc;
use log::{info, error, warn};
use shared::coordinates::Coordinates;
use shared::send_packet;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, Lines};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use shared::messages::{AnalyticsField, AnalyticsPeriodMessage, Message};
use shared::update_position::UpdatePosition;

use crate::server_state::ServerState;
use crate::analytics::{
    analyze_movement, AnalysisPeriod, AnalyticsConfig, MovementStatistics,
};



// Gestisce l'intero ciclo di vita di un client connesso al server:

// 1. Autenticazione (login o registrazione).
// 2. Sessione attiva (task di scrittura e ascolto messaggi).
// 3. Disconnessione e pulizia delle risorse.

pub async fn handle_client(socket: TcpStream, state: Arc<ServerState>) {

    let (read_half, mut write_half) = socket.into_split();
    let mut reader = BufReader::new(read_half).lines();
    let (username, rx) = match authenticate_client(&mut reader, &mut write_half, &state).await {
        
        Some(pair) => pair,
        None => return,

    };

    info!("[AUTH]\t\tUser '{}' successfully authenticated", username);

    let writer_task = setup_active_session(write_half, rx).await;

    process_client_messages(&mut reader, &state, &username).await;
    disconnect_client(&username, &state, writer_task).await;
}



// Questa funzione gestisce la fase iniziale in cui il client deve farsi riconoscere.
// Legge i pacchetti in arrivo e interroga lo stato centrale per registrare o loggare l'utente.
// In caso di login riuscito, recupera lo storico salvato su disco e inizializza il canale di memoria.

async fn authenticate_client(reader: &mut Lines<BufReader<OwnedReadHalf>>, write_half: &mut OwnedWriteHalf, state: &Arc<ServerState>) -> Option<(String, mpsc::Receiver<Message>)> {
    
    loop {

        let line = match reader.next_line().await {

            Ok(Some(line)) => line,
            _ => {
                info!("[NETWORK]\tA client disconnected during the authentication phase");
                return None;
            }

        };

        let msg: Message = match serde_json::from_str(&line) {

            Ok(m) => m,
            Err(_) => continue,

        };

        match msg {

            Message::Register { username, password } => {

                let response = match state.register(&username, &password).await {
                    
                    Ok(()) => {

                        if let Err(e) = state.save_accounts().await {

                            error!("[ERROR]\t\tError saving accounts: {e}");

                        }

                        Message::RegisterOk
                    }

                    Err(e) => Message::RegisterErr(e.to_string()),
                };

                let _ = send_packet(&mut *write_half, &response).await;
            
            }

            Message::Login { username, password } => {

                if let Err(e) = state.authenticate(&username, &password).await {

                    let _ = send_packet(&mut *write_half, &Message::LoginErr(e.to_string())).await;
                    continue;
                
                }

                let (tx, rx) = mpsc::channel::<Message>(32);
                let login_result = state.try_login(&username, tx).await;

                match login_result {

                    Ok(()) => {
                        match ServerState::load_user_history(&username).await {
                            Ok(history) => {
                                state.restore_user_history(&username, history).await;
                            }
                            Err(error) => {
                                error!("[ERROR]\t\tUnable to load coordinates for {}: {}", username, error);
                            }
                        }

                        let _ = send_packet(&mut *write_half, &Message::LoginOk).await;
                        return Some((username, rx));
                    }

                    Err(msg) => {
                        let _ = send_packet(&mut *write_half, &Message::LoginErr(msg)).await;
                    }
                }
            }

            _ => {
                let _ = send_packet(&mut *write_half, &Message::LoginErr("You must login first.".to_string())).await;
            }
        }
    }
}



// Resta costantemente in attesa di messaggi depositati nel canale mpsc.

async fn setup_active_session(write_half: OwnedWriteHalf, mut rx: mpsc::Receiver<Message>) -> JoinHandle<()> {

    tokio::spawn(async move {
        
        let mut buffered_writer = BufWriter::new(write_half);

        while let Some(msg) = rx.recv().await {

            let _ = send_packet(&mut buffered_writer, &msg).await;
            let _ = buffered_writer.flush().await; 

        }
    })
}



// Preleva le richieste di analisi, i messaggi di chat e le nuove coordinate geografiche
// inviate dal client, spostando l'elaborazione allo stato centrale del server.

async fn process_client_messages(reader: &mut Lines<BufReader<OwnedReadHalf>>, state: &Arc<ServerState>, username: &str) {

    while let Ok(Some(line)) = reader.next_line().await {

        if let Ok(msg) = serde_json::from_str::<Message>(&line) {

            match msg {

                Message::AnalyticsRequest { field, period } => {

                    let response = handle_analytics_request(username, field, period, state).await;
                    send_to_logged_user(username, response, state).await;
                
                }

                Message::SendDirectMessage { to, text } => {

                    let _ = state.direct_message(
                        &to, 
                        Message::IncomingDirectMessage { from: username.to_string(), text }
                    ).await;

                }

                Message::SendBroadcastMessage { text } => {

                    state.broadcast(
                        Message::IncomingBroadcastMessage { from: username.to_string(), text }
                    ).await;
                
                }

                _ => {}
            }
            continue;
        }

        if let Ok(update_position) = serde_json::from_str::<UpdatePosition>(&line) {

            if update_position.username != username {

                warn!("[SECURITY]\tPacket rejected: user '{}' attempted to send positions for '{}'", username, update_position.username);
                continue;

            }

            info!("[TRACKER]\t[{}] Coordinates update: lat={:.5}, lon={:.5}, timestamp={}",
                update_position.username,
                update_position.coordinates.get_latitude(),
                update_position.coordinates.get_longitude(),
                update_position.time
            );

            if let Err(error) = ServerState::save_position(username, &update_position).await {
                error!("[ERROR]\t\tUnable to save coordinates for {}: {}", username, error);
            }

            state.process_packet(update_position).await;
            
            continue;
        }

        error!("[ERROR]\t\tUnknown or corrupted packet received from '{}': {}", username, line);
    }
}



// Permette di far recapitare un pacchetto specifico a un utente loggato.

async fn send_to_logged_user(username: &str, msg: Message, state: &Arc<ServerState>) {

    let tx = {

        let connections = state.connections.read().await;
        connections.get(username).cloned()

    };
    
    if let Some(tx) = tx {

        let _ = tx.send(msg).await;

    }
}



// Converte l'intervallo temporale scelto dall'utente nel formato interno di 
// enumerazione richiesto dal motore di statistica (Analytics).

fn convert_period(period: AnalyticsPeriodMessage) -> AnalysisPeriod {

    match period {

        AnalyticsPeriodMessage::CurrentDay => AnalysisPeriod::CurrentDay,
        AnalyticsPeriodMessage::CurrentWeek => AnalysisPeriod::CurrentWeek,
        AnalyticsPeriodMessage::CurrentMonth => AnalysisPeriod::CurrentMonth,
        AnalyticsPeriodMessage::Custom { start_timestamp, end_timestamp } => {
            
            AnalysisPeriod::Custom { start_timestamp, end_timestamp }
        
        }
    }
}



// Trasforma i risultati numerici calcolati dal server in una stringa di testo.

fn format_analytics_response(field: AnalyticsField, stats: &MovementStatistics) -> String {
    
    match field {

        AnalyticsField::Path => {

            let path = stats.path.iter()
                .map(|sample| format!("({:.5}, {:.5})", sample.get_latitude(), sample.get_longitude()))
                .collect::<Vec<String>>().join(" -> ");
            format!("Path taken:\n{}", path)
        }

        AnalyticsField::TotalDistance => format!("Total distance traveled: {:.3} km", stats.total_distance_km),

        AnalyticsField::AverageSpeed => format!("Average speed: {:.3} km/h", stats.average_speed_kmh),

        AnalyticsField::MovementDuration => format!("Total movement duration: {} seconds", stats.movement_duration.as_secs()),

        AnalyticsField::PauseDuration => format!("Total pause duration: {} seconds", stats.pause_duration.as_secs()),

        AnalyticsField::All => {

            let path = stats.path.iter()
                .map(|sample| format!("({:.5}, {:.5})", sample.get_latitude(), sample.get_longitude()))
                .collect::<Vec<String>>().join(" -> ");

            format!(
                "Path taken:\n{}\n\nTotal distance: {:.3} km\nAverage speed: {:.3} km/h\nMovement duration: {} sec\nPause duration: {} sec",
                path, stats.total_distance_km, stats.average_speed_kmh, stats.movement_duration.as_secs(), stats.pause_duration.as_secs()
            )

        }
    }
}



// Gestione di una richiesta di analytics

async fn handle_analytics_request(username: &str, field: AnalyticsField, period: AnalyticsPeriodMessage, state: &Arc<ServerState>) -> Message {

    let history = {

        let users = state.users.read().await;

        match users.get(username) {

            Some(tracker) => tracker.get_history().clone(),
            None => return Message::AnalyticsErr("No positions available for this user.".to_string()),

        }
    };

    if history.is_empty() {

        return Message::AnalyticsErr("Empty position history for this user.".to_string());

    }

    let samples: Vec<Coordinates> = history.iter().map(|update| update.coordinates.clone()).collect();
    let analysis_period = convert_period(period);
    let stats = analyze_movement(&samples, analysis_period, AnalyticsConfig::default());
    let response = format_analytics_response(field, &stats);

    Message::AnalyticsResponse(response)

}



// Termina la connessione dal client.

async fn disconnect_client(username: &str, state: &Arc<ServerState>, writer_task: JoinHandle<()>) {

    if let Some(tracker) = state.users.write().await.get_mut(username) {

        tracker.set_disconnected();

    }
    
    state.logout(username).await;
    writer_task.abort();
    info!("[NETWORK]\t[{}] DISCONNECTED", username);
}