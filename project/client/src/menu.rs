use tokio::sync::mpsc;
use shared::messages::Message;

use crate::utils::read_line_trimmed; 
use crate::utils::Outgoing;

pub enum MenuAction {
    Logout,
}



// Mostra le opzioni disponibili all'utente loggato e smista le relative azioni.
// Legge la scelta da terminale, simula le elaborazioni sul tragitto o intercetta
// l'invio di messaggi diretti e broadcast incanalandoli verso il task del server.
pub async fn run_main_menu(tx: &mpsc::Sender<Outgoing>, username: &str) -> MenuAction {
    println!("\n=== Welcome, {username}! ===");

    loop {
        println!("\n--- Menu ---");
        println!("1) Route info");
        println!("2) Average speed");
        println!("3) Movement duration");
        println!("4) Pause duration");
        println!("5) Send message");
        println!("6) Logout");
        
        let choice = read_line_trimmed("Select an action: ");

        match choice.as_str() {
            "1" => println!("Calculating route information..."),
            "2" => println!("Calculating average speed..."),
            "3" => println!("Calculating total movement duration..."),
            "4" => println!("Calculating pause duration..."),
            "5" => {
                println!("\nMessage type:");
                println!("1) Direct");
                println!("2) Broadcast");
                println!("3) Back");

                let msg_type = read_line_trimmed("Select an option: ");

                match msg_type.as_str() {
                    "1" => {
                        let recipient = read_line_trimmed("Recipient username: ");
                        let text = read_line_trimmed("Message text: ");
                        
                        let msg = Message::SendDirectMessage { to: recipient, text };
                        if tx.send(Outgoing::Chat(msg)).await.is_err() {
                            eprintln!("Error: connection with writer task lost.");
                        }
                    }
                    "2" => {
                        let text = read_line_trimmed("Broadcast message text: ");
                        
                        let msg = Message::SendBroadcastMessage { text };
                        if tx.send(Outgoing::Chat(msg)).await.is_err() {
                            eprintln!("Error: connection with writer task lost.");
                        }
                    }
                    "3" => continue,
                    _ => println!("Invalid option, returning to main menu."),
                }
            }
            "6" => {
                println!("Logging out...");
                return MenuAction::Logout;
            }
            _ => println!("Invalid choice, please try again."),
        }
    }
}