use tokio::sync::mpsc;

use chrono::{
    Local,
    LocalResult,
    NaiveDate,
    NaiveDateTime,
    TimeZone,
};

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
//
// Il client NON calcola le statistiche.
// Qui costruisce solo la richiesta con:
// - il campo richiesto: tragitto, velocità media, movimento o pause;
// - il periodo temporale scelto dall'utente.
//
// Il calcolo vero viene fatto lato server.
async fn send_analytics_request(
    tx: &mpsc::Sender<Outgoing>,
    field: AnalyticsField,
    period: AnalyticsPeriodMessage,
) {
    let request = Message::AnalyticsRequest {
        field,
        period,
    };

    if tx.send(Outgoing::Chat(request)).await.is_err() {
        eprintln!("Impossibile inviare la richiesta analytics: connessione con il server interrotta.");
    }
}



// Menu principale post-login.

// Chiede all'utente su quale periodo vuole fare l'analisi.
//
// Ritorna Some(period) se l'utente sceglie un periodo valido.
// Ritorna None se l'utente decide di tornare al menu principale.
async fn choose_analytics_period() -> Option<AnalyticsPeriodMessage> {
    loop {
        println!("\n--- Analysis period ---");
        println!("1) All available history");
        println!("2) Current day");
        println!("3) Current week");
        println!("4) Current month");
        println!("5) Custom range");
        println!("6) Back");

        let choice = read_line_async("Select a period: ".to_string()).await;

        match choice.as_str() {
            "1" => {
                // Analizza tutto lo storico disponibile.
                // È lo stesso comportamento che avevamo prima.
                return Some(AnalyticsPeriodMessage::Custom {
                    start_timestamp: 0,
                    end_timestamp: i64::MAX,
                });
            }

            "2" => {
                return Some(AnalyticsPeriodMessage::CurrentDay);
            }

            "3" => {
                return Some(AnalyticsPeriodMessage::CurrentWeek);
            }

            "4" => {
                return Some(AnalyticsPeriodMessage::CurrentMonth);
            }

            "5" => {
                return read_custom_period().await;
            }

            "6" => {
                return None;
            }

            _ => {
                println!("Invalid period, please try again.");
            }
        }
    }
}


// Permette all'utente di inserire un intervallo temporale personalizzato.
//
// Formati accettati:
// - 2026-10-10
// - 2026-10-10 09:30
// - 2026-10-10 09:30:00
// - 10/10/2026
// - 10/10/2026 09:30
// - 10-10-2026
// - 10-10-2026 09:30
//
// Se l'utente inserisce solo la data:
// - la data iniziale viene interpretata come 00:00:00;
// - la data finale viene interpretata come 23:59:59.
async fn read_custom_period() -> Option<AnalyticsPeriodMessage> {
    loop {
        println!("\nCustom range examples:");
        println!("Start date/time: 2026-10-10");
        println!("End date/time:   2026-11-20");
        println!("Or with hours:   2026-10-10 09:30");
        println!("Type 'back' to return to the main menu.");

        let start_input = read_line_async("Start date/time: ".to_string()).await;

        if start_input.trim().eq_ignore_ascii_case("back") {
            return None;
        }

        let end_input = read_line_async("End date/time: ".to_string()).await;

        if end_input.trim().eq_ignore_ascii_case("back") {
            return None;
        }

        let start_timestamp = match parse_user_datetime(&start_input, false) {
            Some(timestamp) => timestamp,
            None => {
                println!("Invalid start date/time format. Please try again.");
                continue;
            }
        };

        let end_timestamp = match parse_user_datetime(&end_input, true) {
            Some(timestamp) => timestamp,
            None => {
                println!("Invalid end date/time format. Please try again.");
                continue;
            }
        };

        if start_timestamp > end_timestamp {
            println!("Invalid range: start date/time must be before end date/time.");
            continue;
        }

        return Some(AnalyticsPeriodMessage::Custom {
            start_timestamp,
            end_timestamp,
        });
    }
}


// Converte una data/ora inserita dall'utente in timestamp Unix.
//
// is_end serve solo quando l'utente inserisce una data senza orario:
// - per l'inizio usiamo 00:00:00;
// - per la fine usiamo 23:59:59.
fn parse_user_datetime(input: &str, is_end: bool) -> Option<i64> {
    let input = input.trim();

    let datetime_formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%d/%m/%Y %H:%M:%S",
        "%d/%m/%Y %H:%M",
        "%d-%m-%Y %H:%M:%S",
        "%d-%m-%Y %H:%M",
    ];

    for format in datetime_formats {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(input, format) {
            return local_datetime_to_timestamp(datetime);
        }
    }

    let date_formats = [
        "%Y-%m-%d",
        "%d/%m/%Y",
        "%d-%m-%Y",
    ];

    for format in date_formats {
        if let Ok(date) = NaiveDate::parse_from_str(input, format) {
            let datetime = if is_end {
                date.and_hms_opt(23, 59, 59)?
            } else {
                date.and_hms_opt(0, 0, 0)?
            };

            return local_datetime_to_timestamp(datetime);
        }
    }

    None
}


// Trasforma una data/ora locale in timestamp Unix.
//
// L'utente inserisce date e orari come orari locali.
// Il timestamp Unix ottenuto viene poi inviato al server.
fn local_datetime_to_timestamp(datetime: NaiveDateTime) -> Option<i64> {
    match Local.from_local_datetime(&datetime) {
        LocalResult::Single(local_datetime) => Some(local_datetime.timestamp()),

        // Caso raro del cambio ora legale/solare:
        // lo stesso orario locale può corrispondere a due istanti diversi.
        // Prendiamo il primo per evitare di bloccare il programma.
        LocalResult::Ambiguous(first_datetime, _) => Some(first_datetime.timestamp()),

        // Caso raro: orario inesistente per cambio ora legale.
        LocalResult::None => None,
    }
}


// Funzione di supporto per non ripetere la scelta del periodo
// nei casi 1, 2, 3 e 4 del menu.
async fn ask_period_and_send_analytics_request(
    tx: &mpsc::Sender<Outgoing>,
    field: AnalyticsField,
) {
    match choose_analytics_period().await {
        Some(period) => {
            send_analytics_request(tx, field, period).await;
        }

        None => {
            println!("Returning to main menu.");
        }
    }
}


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
                ask_period_and_send_analytics_request(tx, AnalyticsField::Path).await;
            }

            "2" => {
                println!("Richiesta velocità media al server...");
                ask_period_and_send_analytics_request(tx, AnalyticsField::AverageSpeed).await;
            }

            "3" => {
                println!("Richiesta durata complessiva del movimento al server...");
                ask_period_and_send_analytics_request(tx, AnalyticsField::MovementDuration).await;
            }

            "4" => {
                println!("Richiesta durata complessiva delle pause al server...");
                ask_period_and_send_analytics_request(tx, AnalyticsField::PauseDuration).await;
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