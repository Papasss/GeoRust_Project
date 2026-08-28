use tokio::fs::File;
use std::io;
use std::io::ErrorKind;

pub mod messages;
pub mod coordinates;
pub mod user_state;
pub mod update_position;
pub mod utils;



// Apre un file in modalità completamente asincrona partendo dal percorso specificato.
// Intercetta eventuali errori di sistema, stampando un messaggio diagnostico 
// prima di restituire l'errore al chiamante.
pub async fn read_file(path: &str) -> io::Result<File> {
    match File::open(path).await {
        Ok(f) => Ok(f),
        Err(e) => {
            match e.kind() {
                ErrorKind::PermissionDenied => eprintln!("Permission denied"),
                ErrorKind::NotFound => eprintln!("File does not exist"),
                _ => eprintln!("Generic error: {}", e),
            } 
            Err(e)
        }
    }
}



// Suddivide una stringa di testo nei suoi elementi costitutivi.
// Utilizza spazi bianchi, virgole o punti e virgola come separatori, 
// scartando elementi vuoti e restituendo un vettore di stringhe pulite.
pub fn parse_values(line: &str) -> Vec<String> {
    line
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}