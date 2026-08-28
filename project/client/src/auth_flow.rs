use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use shared::messages::Message;
use shared::utils::send_packet;

use crate::utils::read_line_trimmed;



// Legge la password digitata dall'utente oscurando i caratteri a schermo.
// Utilizza la libreria rpassword per garantire la privacy e la sicurezza durante
// l'inserimento delle credenziali dirette nel terminale.
fn read_password(prompt: &str) -> String {
    rpassword::prompt_password(prompt).expect("Error reading password")
}



// Legge una singola riga dal socket TCP e tenta di convertirla in una struttura Message.
// Restituisce il messaggio deserializzato, oppure None se la connessione
// viene chiusa dal server o se si verifica un errore di lettura e formato.
async fn read_response(stream: &mut TcpStream) -> Option<Message> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    match reader.read_line(&mut line).await {
        Ok(0) | Err(_) => None, 
        Ok(_) => serde_json::from_str(line.trim()).ok(),
    }
}



// Gestisce l'intero ciclo interattivo di registrazione e login dell'utente.
// Mostra le opzioni a schermo, raccoglie le credenziali inviandole asincronamente al server
// e ripete le operazioni finché l'autenticazione non va a buon fine, restituendo lo username.
pub async fn run_auth_flow(stream: &mut TcpStream) -> String {
    loop {
        println!("\n=== GEORUST ===");
        println!("1) Register");
        println!("2) Login");
        println!("3) Exit");
        
        let choice = read_line_trimmed("Select an option: ");

        match choice.as_str() {
            "1" => {
                let username = read_line_trimmed("Username: ");
                let password = read_password("Password: ");

                let _ = send_packet(stream, &Message::Register { username, password }).await;

                match read_response(stream).await {
                    Some(Message::RegisterOk) => {
                        println!("Registration successful! You can now log in.");
                    }
                    Some(Message::RegisterErr(msg)) => {
                        println!("Registration failed: {}", msg);
                    }
                    Some(_) => println!("Unexpected response from server."),
                    None => {
                        eprintln!("Connection lost with the server.");
                        std::process::exit(1);
                    }
                }
            }
            "2" => {
                let username = read_line_trimmed("Username: ");
                let password = read_password("Password: ");

                let _ = send_packet(stream, &Message::Login { username: username.clone(), password }).await;

                match read_response(stream).await {
                    Some(Message::LoginOk) => {
                        println!("Login successful! Welcome, {}.", username);
                        return username; 
                    }
                    Some(Message::LoginErr(msg)) => {
                        println!("Login failed: {}", msg);
                    }
                    Some(_) => println!("Unexpected response from server."),
                    None => {
                        eprintln!("Connection lost with the server.");
                        std::process::exit(1);
                    }
                }
            }
            "3" => {
                println!("Exiting...");
                std::process::exit(0);
            }
            _ => println!("Invalid option, please try again."),
        }
    }
}