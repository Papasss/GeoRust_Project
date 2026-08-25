use tokio::net::TcpStream;
use tokio::sync::mpsc;

use shared::messages::Message;

use crate::send_to_server;
use crate::read_line_trimmed; 
use crate::Outgoing;

pub enum MenuAction {
    Logout,
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
                println!("Calcolo informazioni sul tragitto percorso..,");
            }
            "2" => {
                println!("Calcolo velocità media...");
            }
            "3" => {
                println!("Calcolo durata complessiva del movimento...");
            }
            "4" => {
                println!("Calcolo durata delle pause...");
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