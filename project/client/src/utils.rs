use std::io::{self, Write};

use chrono::DateTime;
use shared::{coordinates::Coordinates, messages::Message, parse_values, update_position::UpdatePosition};
use tokio::io::AsyncBufReadExt;



pub enum Outgoing {
    Chat(Message),
    Position(UpdatePosition),
}



// Estrae i dati dal file di testo caricandoli in memoria per il futuro invio temporizzato.
// Apre il file in modo asincrono, lo legge riga per riga per non bloccare il processo, 
// e costruisce e restituisce il vettore completo di coordinate associate all'utente.
pub async fn load_user_path(file_path: &str, username: &str) -> io::Result<Vec<UpdatePosition>> {
    let mut route: Vec<UpdatePosition> = Vec::new();
    
    let file = tokio::fs::File::open(file_path).await?;
    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let values = parse_values(&line);

        if values.is_empty() { continue; }

        let coordinates = Coordinates::new(values[0].clone(), values[1].clone());
        let time = DateTime::parse_from_rfc3339(&values[2])
            .map(|time| time.with_timezone(&chrono::Utc))
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

        let update_position = UpdatePosition {
            username: username.to_string(),
            coordinates,
            time
        };

        route.push(update_position);
    }

    Ok(route)
}

// Legge una riga di testo in input dal terminale bloccando temporaneamente l'esecuzione.
// Mostra il prompt richiesto, attende la digitazione dell'utente e
// restituisce la stringa pulita da eventuali spazi o ritorni a capo esterni.
pub fn read_line_trimmed(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Error reading input");
    input.trim().to_string()
}