use std::{io::{self, Write}, time::Duration};

use chrono::DateTime;
use shared::{coordinates::Coordinates, messages::Message, parse_values, update_position::UpdatePosition, send_packet};
use tokio::{io::{AsyncBufReadExt, BufReader, Lines}, net::tcp::{OwnedReadHalf, OwnedWriteHalf}, task::JoinHandle, sync::{mpsc}};


pub enum Outgoing {

    Chat(Message),
    Position(UpdatePosition),

}



// Estrae i dati e restituisce il vettore completo di coordinate associate 
// all'utente.

pub async fn load_user_path(file_path: &str, username: &str) -> io::Result<Vec<UpdatePosition>> {
   
    let mut route: Vec<UpdatePosition> = Vec::new();
    let file = tokio::fs::File::open(file_path).await?;
    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {

        let values = parse_values(&line);

        if values.is_empty() { continue; }

        
        let time = DateTime::parse_from_rfc3339(&values[2])
            .map(|time| time.with_timezone(&chrono::Utc))
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let coordinates = Coordinates::new(values[0].clone(), values[1].clone(), time.timestamp());
        
        let update_position = UpdatePosition {
            username: username.to_string(),
            coordinates,
            time
        };

        route.push(update_position);
    }

    Ok(route)

}



// Legge una riga di testo in input dal terminale in modo asincrono.

pub async fn read_line_async(prompt: String) -> String {

    tokio::task::spawn_blocking(move || {

        print!("{prompt}");

        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Error reading input");
        
        input.trim().to_string()

    }).await.expect("Error in I/O Thread!")

}



// Avvia un task in background dedicato esclusivamente all'ascolto dei messaggi in arrivo.

pub fn spawn_reader_task(mut reader: Lines<BufReader<OwnedReadHalf>>) -> JoinHandle<()> {

    tokio::spawn(async move {

        while let Ok(Some(line)) = reader.next_line().await {

            if let Ok(msg) = serde_json::from_str::<Message>(&line) { 

                match msg {

                    Message::IncomingDirectMessage { from, text } => {
                        println!("\n[\x1b[36mPrivate message from {}\x1b[0m]: {}", from, text);
                    }

                    Message::IncomingBroadcastMessage { from, text } => {
                        println!("\n[\x1b[33mBroadcast from {}\x1b[0m]: {}", from, text);
                    }

                    Message::AnalyticsResponse(response) => {
                        println!("\n\x1b[32m╔════════════════════════════════════╗\x1b[0m");
                        println!("\x1b[32m║         ANALYTICS RESULT           ║\x1b[0m");
                        println!("\x1b[32m╚════════════════════════════════════╝\x1b[0m");
                        println!("{response}\n");
                    }

                    Message::AnalyticsErr(error) => {
                        eprintln!("\n\x1b[31mAnalytics Error:\x1b[0m {error}\n");
                    }

                    _ => {} 
                }
            }
        }
    })
}



// Riceve i messaggi dal canale MPSC e li trasmette sulla rete.

pub fn spawn_writer_task(mut rx: mpsc::Receiver<Outgoing>, mut write_half: OwnedWriteHalf) -> JoinHandle<()> {

    tokio::spawn(async move {

        while let Some(packet) = rx.recv().await {

            match packet {

                Outgoing::Chat(msg) => { let _ = send_packet(&mut write_half, &msg).await; },
                Outgoing::Position(upd) => { let _ = send_packet(&mut write_half, &upd).await; },
            
            }
        }
    })
}



// Esegue l'invio temporizzato delle coordinate geografiche dell'utente ogni 30 secondi.
// Riprende dall'ultima posizione conosciuta.

pub fn spawn_position_task(
    file_path: String,
    tx: mpsc::Sender<Outgoing>,
    username: String,
    last_position: Option<(f64, f64)>
) -> JoinHandle<()> {

    tokio::spawn(async move {
        
        let mut previous_position = last_position;

        loop {
            
            tokio::time::sleep(Duration::from_secs(30)).await;

            let position = match crate::path_manager::append_random_position(
                &file_path,
                &username,
                previous_position,
            )
            .await
            {
                Ok(position) => position,
                Err(error) => {
                    eprintln!("Unable to save position: {error}");
                    break;
                }
            };

            previous_position = Some(position.0);
            
            if tx.send(Outgoing::Position(position.1)).await.is_err() {
                break;
            }
        }

        println!("\n\x1b[32mRoute generation stopped for user {}.\x1b[0m", username);

    })
}