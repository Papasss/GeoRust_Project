use shared::read_file;
use shared::parse_values;
use shared::updatePosition::UpdatePosition;
use shared::coordinates::Coordinates;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader};
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio::time::Duration;
use serde::Serialize;
use std::fmt::Debug;


async fn send_to_server<T>(stream: &mut TcpStream, pacchetto: &T) where T: Serialize + Debug{
    let json_data = serde_json::to_string(pacchetto).expect("Errore nella conversione in JSON");

    if let Err(e) = stream.write_all(format!("{}\n", json_data).as_bytes()).await {
        eprintln!("Errore durante l'invio dei dati al server: {}", e);
    } else {
        println!("[OK] Inviato: {:?}", pacchetto);
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {

    println!("Starting Client Application...");

    let mut stream = match TcpStream::connect("127.0.0.1:8080").await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
            return Ok(());
        }
    };
    println!("Connect to server!");

    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // --- FASE DI AUTENTICAZIONE ---
    let username = auth_flow::run_auth_flow(&mut stream, &mut reader);

    println!("Authenticated successfully as: {}", username);

    // --- FASE DI INVIO COORDINATE ---
    let mut connections: HashMap<String, UpdatePosition> = HashMap::new();

    let file = read_file("input.txt")?;
    let reader = BufReader::new(file);

    for line_result in reader.lines() {
        let line = line_result?;
        let values = parse_values(&line);

        if values.is_empty() {
            continue;
        }

        let username_file = values[0].clone();
        let coordinates  = Coordinates::new(values[1].clone(), values[2].clone());
        let time = values[3].clone();

        let update_position = UpdatePosition {
            username: username_file.clone(),
            coordinates: coordinates.clone(),
            time: time.clone()
        };

        connections.insert(username_file, update_position.clone());
    }

    if connections.is_empty() {
        println!("Nessuna posizione trovata in input.txt");
        return Ok(());
    }

    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;

        match connections.get(&username) {
            Some(upd) => {
                send_to_server(&mut stream, upd).await;
            }
            None => {
                println!("Nessuna posizione trovata per l'utente autenticato: {}", username);
            }
        }
    }

}




