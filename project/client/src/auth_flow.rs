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

        // Pulisce lo schermo e sposta il cursore in alto a sinistra
        print!("\x1B[2J\x1B[1;1H");
        
        println!("╔════════════════════════════════════╗");
        println!("║             GEORUST                ║");
        println!("╚════════════════════════════════════╝");
        println!("1) Register");
        println!("2) Login");
        println!("3) Exit");
        println!("──────────────────────────────────────");
        
        let choice = read_line_async("Select an option: ".to_string()).await;

        match choice.as_str() {

            "1" => {

                let username = read_line_async("Username: ".to_string()).await;
                let password = read_password("Password: ");
                let _ = send_packet(stream, &Message::Register { username, password }).await;

                match read_response(stream).await {

                    Some(Message::RegisterOk) => {
                        println!("\n\x1b[32mRegistration successful! You can now log in.\x1b[0m");
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }

                    Some(Message::RegisterErr(msg)) => {
                        println!("\n\x1b[31mRegistration failed: {}\x1b[0m", msg);
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }

                    Some(_) => {
                        println!("\n\x1b[31mUnexpected response from server.\x1b[0m");
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }

                    None => {
                        eprintln!("\n\x1b[31mConnection lost with the server.\x1b[0m");
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
                        println!("\n\x1b[32mLogin successful! Welcome, {}.\x1b[0m", username);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        return username; 
                    }

                    Some(Message::LoginErr(msg)) => {
                        println!("\n\x1b[31mLogin failed: {}\x1b[0m", msg);
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }

                    Some(_) => {
                        println!("\n\x1b[31mUnexpected response from server.\x1b[0m");
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }

                    None => {
                        eprintln!("\n\x1b[31mConnection lost with the server.\x1b[0m");
                        std::process::exit(1);
                    }
                }
            }

            "3" => {

                println!("Exiting...");
                std::process::exit(0);

            }
            
            _ => {
                println!("\n\x1b[33mInvalid option, please try again.\x1b[0m");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}