use tokio::sync::mpsc;

use shared::messages::{
    AnalyticsField,
    AnalyticsPeriodMessage,
    Message,
};

use crate::utils::Outgoing;
use crate::utils::read_line_async;



pub enum MenuAction {

    Logout,

}


// Invia al server una richiesta di analytics.

async fn send_analytics_request(tx: &mpsc::Sender<Outgoing>, field: AnalyticsField) {
    
    let request = Message::AnalyticsRequest {
        field,
        period: AnalyticsPeriodMessage::Custom {
            start_timestamp: 0,
            end_timestamp: i64::MAX,
        },
    };

    if tx.send(Outgoing::Chat(request)).await.is_err() {
        eprintln!("Impossibile inviare la richiesta analytics: connessione con il server interrotta.");
    }
}



// Menu principale post-login.

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
                        "1" => {
                println!("Richiesta tragitto percorso al server...");
                send_analytics_request(tx, AnalyticsField::Path).await;
            }

            "2" => {
                println!("Richiesta velocità media al server...");
                send_analytics_request(tx, AnalyticsField::AverageSpeed).await;
            }

            "3" => {
                println!("Richiesta durata complessiva del movimento al server...");
                send_analytics_request(tx, AnalyticsField::MovementDuration).await;
            }

            "4" => {
                println!("Richiesta durata complessiva delle pause al server...");
                send_analytics_request(tx, AnalyticsField::PauseDuration).await;
            }
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