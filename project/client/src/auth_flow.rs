use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use shared::messages::Message;
use shared::send_packet;

use crate::utils::read_line_async;



// Legge la password digitata dall'utente oscurando i caratteri a schermo.

fn read_password(prompt: &str) -> String {

    rpassword::prompt_password(prompt).expect("Error reading password")

}



// Attende la risposta di registrazione / login in arrivo dal server

async fn read_response(stream: &mut TcpStream) -> Option<Message> {

    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    match reader.read_line(&mut line).await {

        Ok(0) | Err(_) => None, 
        Ok(_) => serde_json::from_str(line.trim()).ok(),

    }
}



// Gestisce l'intero ciclo di registrazione e login dell'utente.

pub async fn register_and_login(stream: &mut TcpStream) -> String {

    loop {

        println!("\n=== GEORUST ===");
        println!("1) Register");
        println!("2) Login");
        println!("3) Exit");
        
        let choice = read_line_async("Select an option: ".to_string()).await;

        match choice.as_str() {

            "1" => {

                let username = read_line_async("Username: ".to_string()).await;
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

                let username = read_line_async("Username: ".to_string()).await;
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