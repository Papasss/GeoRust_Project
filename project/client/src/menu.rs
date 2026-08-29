use tokio::sync::mpsc;
use shared::messages::Message;

use crate::utils::read_line_async; 
use crate::utils::Outgoing;

pub enum MenuAction {

    Logout,

}



// Mneù interattivo del client una volta loggati

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
        
        let choice = read_line_async("Select an action: ".to_string()).await;

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

                let msg_type = read_line_async("Select an option: ".to_string()).await;

                match msg_type.as_str() {

                    "1" => {

                        let recipient = read_line_async("Recipient username: ".to_string()).await;
                        let text = read_line_async("Message text: ".to_string()).await;
                        let msg = Message::SendDirectMessage { to: recipient, text };

                        if tx.send(Outgoing::Chat(msg)).await.is_err() {

                            eprintln!("Error: connection with writer task lost.");

                        }
                    }

                    "2" => {

                        let text = read_line_async("Broadcast message text: ".to_string()).await;
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