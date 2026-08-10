use shared::coordinates;
use shared::read_file;
use shared::parse_values;
use shared::updatePosition::UpdatePosition;
use shared::coordinates::Coordinates;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader};
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;


// Funzione di supporto per gestire la connessione di rete
async fn invia_al_server(pacchetto: &UpdatePosition) {
    // Tentiamo la connessione al server (assumendo che sia in ascolto sulla porta 8080 in locale)
    match TcpStream::connect("127.0.0.1:8080").await {
        Ok(mut stream) => {
            // Serializziamo la struct in una stringa JSON
            let dati_json = serde_json::to_string(pacchetto).expect("Errore nella conversione in JSON");

            if let Err(e) = stream.write_all(format!("{}\n", dati_json).as_bytes()).await {
                eprintln!("Errore durante l'invio dei dati al server: {}", e);
            } else {
                println!("[OK] Inviato: {:?}", pacchetto);
            }
        }
        Err(e) => {
            eprintln!("Impossibile connettersi al server ({}).", e);
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {

    let mut connections = HashMap::new();

    let file = read_file("input.txt")?;
    let reader = BufReader::new(file);

    for line_result in reader.lines() {
        let line = line_result?;
        let values = parse_values(&line);

        if values.is_empty() {
            continue;
        }

        let username = values[0].clone();
        let coordinates  = Coordinates::new(values[1].clone(), values[2].clone());
        let time = values[3].clone();

        let update_position = UpdatePosition {
            username: username.clone(),
            coordinates: coordinates.clone(),
            time: time.clone()
        };

        connections.insert(username, update_position.clone());

        invia_al_server(&update_position).await;

    }

    Ok(())

    

}


