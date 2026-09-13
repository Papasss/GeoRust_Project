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



// Menu principale post-login.

async fn choose_analytics_period() -> Option<AnalyticsPeriodMessage> {

    loop {

        print!("\x1B[2J\x1B[1;1H");
        println!("╔════════════════════════════════════╗");
        println!("║          ANALYSIS PERIOD           ║");
        println!("╚════════════════════════════════════╝");
        println!("1) All available history");
        println!("2) Current day");
        println!("3) Current week");
        println!("4) Current month");
        println!("5) Custom range");
        println!("6) Back");
        println!("──────────────────────────────────────");

        let choice = read_line_async("Select a period: ".to_string()).await;

        match choice.as_str() {
            
            "1" => {
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
                println!("\x1b[33mInvalid period, please try again.\x1b[0m");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}



// Permette all'utente di inserire un intervallo temporale personalizzato.

async fn read_custom_period() -> Option<AnalyticsPeriodMessage> {

    loop {

        print!("\x1B[2J\x1B[1;1H");
        println!("--- Custom range examples ---");
        println!("Start date/time: 2026-10-10");
        println!("End date/time:   2026-11-20");
        println!("Or with hours:   2026-10-10 09:30");
        println!("Type 'back' to return to the previous menu.");
        println!("──────────────────────────────────────");

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
                println!("\x1b[31mInvalid start date/time format. Please try again.\x1b[0m");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }
        };

        let end_timestamp = match parse_user_datetime(&end_input, true) {
            Some(timestamp) => timestamp,
            None => {
                println!("\x1b[31mInvalid end date/time format. Please try again.\x1b[0m");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }
        };

        if start_timestamp > end_timestamp {
            println!("\x1b[31mInvalid range: start date/time must be before end date/time.\x1b[0m");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            continue;
        }

        return Some(AnalyticsPeriodMessage::Custom {
            start_timestamp,
            end_timestamp,
        });
    }
}



// Converte una data/ora inserita dall'utente in timestamp Unix.

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

fn local_datetime_to_timestamp(datetime: NaiveDateTime) -> Option<i64> {

    match Local.from_local_datetime(&datetime) {
        LocalResult::Single(local_datetime) => Some(local_datetime.timestamp()),
        LocalResult::Ambiguous(first_datetime, _) => Some(first_datetime.timestamp()),
        LocalResult::None => None,
    }
}



// Funzione unificata per non ripetere la scelta del periodo.

async fn ask_period_and_send_analytics_request(
    tx: &mpsc::Sender<Outgoing>,
    field: AnalyticsField,
) {

    match choose_analytics_period().await {
        
        Some(period) => {
            
            let request = Message::AnalyticsRequest { field, period };

            if tx.send(Outgoing::Chat(request)).await.is_err() {
                eprintln!("\x1b[31mError: connection with writer task lost.\x1b[0m");
                let _ = read_line_async("\n\x1b[33mPress Enter to return to the main menu...\x1b[0m".to_string()).await;
            } else {
                
                println!("\n\x1b[36mWaiting for server response...\x1b[0m");
                let _ = read_line_async("\n\x1b[33mPress Enter to return to the main menu...\x1b[0m".to_string()).await;
                
            }
        }

        None => {} 
    }
}



// Loop principale dell'interfaccia utente a sessione attiva.

pub async fn run_main_menu(tx: &mpsc::Sender<Outgoing>, username: &str) -> MenuAction {

    loop {

        print!("\x1B[2J\x1B[1;1H");
        println!("=== Welcome, \x1b[32m{}\x1b[0m! ===", username);
        println!("\n╔════════════════════════════════════╗");
        println!("║             MAIN MENU              ║");
        println!("╚════════════════════════════════════╝");
        println!("1) Route info");
        println!("2) Average speed");
        println!("3) Movement duration");
        println!("4) Pause duration");
        println!("5) Send message");
        println!("6) Logout");
        println!("──────────────────────────────────────");
        
        let choice = read_line_async("Select an action: ".to_string()).await;

        match choice.as_str() {
            "1" => {
                ask_period_and_send_analytics_request(tx, AnalyticsField::Path).await;
            }

            "2" => {
                ask_period_and_send_analytics_request(tx, AnalyticsField::AverageSpeed).await;
            }

            "3" => {
                ask_period_and_send_analytics_request(tx, AnalyticsField::MovementDuration).await;
            }

            "4" => {
                ask_period_and_send_analytics_request(tx, AnalyticsField::PauseDuration).await;
            }
            "5" => {
                
                print!("\x1B[2J\x1B[1;1H");
                println!("--- Message type ---");
                println!("1) Direct");
                println!("2) Broadcast");
                println!("3) Back");
                println!("──────────────────────────────────────");

                let msg_type = read_line_async("Select an option: ".to_string()).await;

                match msg_type.as_str() {

                    "1" => {

                        let recipient = read_line_async("Recipient username: ".to_string()).await;
                        let text = read_line_async("Message text: ".to_string()).await;
                        let msg = Message::SendDirectMessage { to: recipient, text };

                        if tx.send(Outgoing::Chat(msg)).await.is_err() {
                            eprintln!("\x1b[31mError: connection with writer task lost.\x1b[0m");
                        } else {
                            println!("\x1b[32mMessage sent successfully!\x1b[0m");
                        }
                        
                        let _ = read_line_async("\n\x1b[33mPress Enter to continue...\x1b[0m".to_string()).await;
                    }

                    "2" => {

                        let text = read_line_async("Broadcast message text: ".to_string()).await;
                        let msg = Message::SendBroadcastMessage { text };

                        if tx.send(Outgoing::Chat(msg)).await.is_err() {
                            eprintln!("\x1b[31mError: connection with writer task lost.\x1b[0m");
                        } else {
                            println!("\x1b[32mBroadcast message sent successfully!\x1b[0m");
                        }
                        
                        let _ = read_line_async("\n\x1b[33mPress Enter to continue...\x1b[0m".to_string()).await;
                    }

                    "3" => continue,

                    _ => {
                        println!("\x1b[33mInvalid option, returning to main menu.\x1b[0m");
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }

                }
            }

            "6" => {

                println!("Logging out...");
                return MenuAction::Logout;

            }
            
            _ => {
                println!("\x1b[33mInvalid choice, please try again.\x1b[0m");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}