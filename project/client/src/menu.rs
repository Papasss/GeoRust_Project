use tokio::sync::mpsc;

use shared::messages::{
    AnalyticsField,
    AnalyticsPeriodMessage,
    Message,
};

use crate::read_line_trimmed;
use crate::Outgoing;

pub enum MenuAction {
    Logout,
}


// Invia al server una richiesta di analytics.
// Per ora analizziamo tutto lo storico disponibile usando un intervallo Custom molto ampio.
// Questo è utile perché il file percorso.txt contiene timestamp fittizi, quindi CurrentDay
// potrebbe filtrare fuori tutte le posizioni.
async fn send_analytics_request(
    tx: &mpsc::Sender<Outgoing>,
    field: AnalyticsField,
) {
    let request = Message::AnalyticsRequest {
        field,
        period: AnalyticsPeriodMessage::Custom {
            start_timestamp: 0,
            end_timestamp: u64::MAX,
        },
    };

    if tx.send(Outgoing::Chat(request)).await.is_err() {
        eprintln!("Impossibile inviare la richiesta analytics: connessione con il server interrotta.");
    }
}

/// Menu principale post-login. Ritorna l'azione scelta dall'utente,
/// così il chiamante (main) decide cosa fare (tornare al login o uscire).
pub async fn run_main_menu(tx: &mpsc::Sender<Outgoing>, username: &str) -> MenuAction {
    println!("\n=== Benvenuto, {username}! ===");

    loop {
        println!("\n--- Menu ---");
        println!("1) Tragitto percorso");
        println!("2) Velocità media");
        println!("3) Durata movimento");
        println!("4) Durata pausa");
        println!("5) Invia messaggio");
        println!("6) Logout");
        let scelta = read_line_trimmed("Seleziona l'azione: ");

        match scelta.as_str() {
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
                let testo = read_line_trimmed("Testo: ");
                if tx.send(Outgoing::Chat(Message::Text(testo))).await.is_err() {
                    eprintln!("Impossibile inviare il messaggio: connessione con il server interrotta.");
                }
            }
            "6" => {
                println!("Logout in corso...");
                return MenuAction::Logout;
            }
            _ => println!("Scelta non valida, riprova."),
        }
    }
}