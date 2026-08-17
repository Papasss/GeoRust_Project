use std::io::{self, Write, BufRead};
use std::net::TcpStream;
use shared::messages::Message;

/// Legge una riga da terminale, con un prompt, e la ripulisce
fn read_line_trimmed(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}
//non visualizzare la password
fn read_password(prompt: &str) -> String {
    rpassword::prompt_password(prompt).expect("Failed to read password")
}
/// Serializza un Message in JSON e lo scrive sul socket, seguito da \n
fn send_request(stream: &mut TcpStream, msg: &Message) {
    let json = serde_json::to_string(msg).unwrap();
    writeln!(stream, "{}", json).expect("Failed to send message to server");
}

/// Legge una riga dal socket e la deserializza in un Message
fn read_response(reader: &mut impl BufRead) -> Option<Message> {
    let mut line = String::new();
    if reader.read_line(&mut line).unwrap_or(0) == 0 {
        return None; // il server ha chiuso la connessione
    }
    serde_json::from_str(line.trim()).ok()
}

/// Gestisce l'intero ciclo di registrazione/login.
/// Ritorna lo username autenticato quando il login ha successo.
pub fn run_auth_flow(stream: &mut TcpStream, reader: &mut impl BufRead) -> String {
    loop {
        println!("\n=== GEORUGGINE ===");
        println!("1) Register");
        println!("2) Login");
        println!("3) Exit");
        let scelta = read_line_trimmed("Select option: ");

        match scelta.as_str() {
            "1" => {
                let username = read_line_trimmed("Username: ");
                let password = read_password("Password: ");

                send_request(stream, &Message::Register { username, password });

                match read_response(reader) {
                    Some(Message::RegisterOk) => {
                        println!("Registration successful! You can now log in.");
                    }
                    Some(Message::RegisterErr(msg)) => {
                        println!("Registration failed: {}", msg);
                    }
                    Some(_) => println!("Unexpected response from server."),
                    None => {
                        eprintln!("Connection lost with server.");
                        std::process::exit(1);
                    }
                }
            }
            "2" => {
                let username = read_line_trimmed("Username: ");
                let password = read_password("Password: ");

                send_request(stream, &Message::Login { username: username.clone(), password });

                match read_response(reader) {
                    Some(Message::LoginOk) => {
                        println!("Login successful! Welcome, {}.", username);
                        return username; // auth completata, usciamo dal loop
                    }
                    Some(Message::LoginErr(msg)) => {
                        println!("Login failed: {}", msg);
                    }
                    Some(_) => println!("Unexpected response from server."),
                    None => {
                        eprintln!("Connection lost with server.");
                        std::process::exit(1);
                    }
                }
            }
            "3" => {
                println!("Exit...");
                std::process::exit(0);
            }
            _ => println!("Invalid option, please try again."),
        }
    }
}