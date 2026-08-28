use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;

use shared::messages::Message;

use crate::send_to_server; 
use crate::read_line_trimmed;

//non visualizzare la password
fn read_password(prompt: &str) -> String {
    rpassword::prompt_password(prompt).expect("Errore durante la lettura della password")
}


/// Legge una riga dal socket e la deserializza in un Message
async fn read_response(stream: &mut TcpStream) -> Option<Message> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    match reader.read_line(&mut line).await {
        Ok(0) | Err(_) => None, // connessione chiusa o errore
        Ok(_) => serde_json::from_str(line.trim()).ok(),
    }
}

/// Gestisce l'intero ciclo di registrazione/login.
/// Ritorna lo username autenticato quando il login ha successo.
pub async fn run_auth_flow(stream: &mut TcpStream) -> String {
    loop {
        println!("\n=== GEORUGGINE ===");
        println!("1) Registrazione");
        println!("2) Login");
        println!("3) Exit");
        let scelta = read_line_trimmed("Seleziona l'opzione: ");

        match scelta.as_str() {
            "1" => {
                let username = read_line_trimmed("Username: ");
                let password = read_password("Password: ");

                send_to_server(stream, &Message::Register { username, password }).await;

                match read_response(stream).await {
                    Some(Message::RegisterOk) => {
                        println!("Registrazione completata con successo! Ora puoi effettuare il login.");
                    }
                    Some(Message::RegisterErr(msg)) => {
                        println!("Registrazione fallita: {}", msg);
                    }
                    Some(_) => println!("Risposta inattesa dal server."),
                    None => {
                        eprintln!("Connessione persa con il server.");
                        std::process::exit(1);
                    }
                }
            }
            "2" => {
                let username = read_line_trimmed("Username: ");
                let password = read_password("Password: ");

                send_to_server(stream, &Message::Login { username: username.clone(), password }).await;

                match read_response(stream).await {
                    Some(Message::LoginOk) => {
                        println!("Login eseguito con successo! Benvenuto, {}.", username);
                        return username; // auth completata, usciamo dal loop
                    }
                    Some(Message::LoginErr(msg)) => {
                        println!("Login fallito: {}", msg);
                    }
                    Some(_) => println!("Risposta inattesa dal server."),
                    None => {
                        eprintln!("Connessione persa con il server.");
                        std::process::exit(1);
                    }
                }
            }
            "3" => {
                println!("Exit...");
                std::process::exit(0);
            }
            _ => println!("Opzione non valida, riprova."),
        }
    }
}