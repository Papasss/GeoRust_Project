use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use shared::messages::Message;
// Il client invia le posizioni come UpdatePosition, non come Message.
// Per questo il server deve riuscire a deserializzare anche questo tipo.
use shared::update_position::UpdatePosition;
use crate::server_state::ServerState;



// Eappresenta l'intero ciclo di vita di un client connesso al server.
// Separa il canale di comunicazione in lettura e scrittura ed orchestra in 
// sequenza le tre fasi pricipali di un client.

pub async fn handle_client(socket: TcpStream, state: Arc<Mutex<ServerState>>) {

    let (read_half, mut write_half) = socket.into_split();
    let mut reader = BufReader::new(read_half).lines();

    // Autenticazione

    let username = match authenticate_client(&mut reader, &mut write_half, &state).await {
        Some(name) => name,
        None => return,
    };

    println!("\t\t\t\tUser authenticated successfully: {username}");

    // Sessione attiva

    println!("\t\t\t\tSetting up user {username} active session...");

    let writer_task = setup_active_session(&username, write_half, &state).await;

    println!("\t\t\t\t\tdone!");

    // Passiamo anche username e stato globale:
    // serviranno nella fase post-login per associare i pacchetti all'utente corretto.
process_client_messages(&username, &mut reader, &state).await;

    // Disconnessione

    disconnect_client(&username, &state, writer_task).await;
}



// Gestisce l'accesso al sistema intercettando i primi messaggi inviati dal 
// client. Elabora richieste di registrazione e login bloccando 
// temporaneamente lo stato del server per i controlli.
// Se un client invia un messaggio diverso prima di aver effettuato l'accesso, 
// riceverà un errore. La funzione continua a ciclare finché l'utente non si 
// autentica con successo o non chiude la connessione.

async fn authenticate_client( reader: &mut Lines<BufReader<OwnedReadHalf>>, write_half: &mut OwnedWriteHalf, state: &Arc<Mutex<ServerState>>) -> Option<String> {
    
    loop {

        let line = match reader.next_line().await {
            Ok(Some(line)) => line,
            _ => {
                println!("\t\t\t\tClient disconnected during authentication phase");
                return None;
            }
        };

        let msg: Message = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(_) => continue,
        };

        match msg {
            Message::Register { username, password } => {
                let mut server_state_lock = state.lock().await;
                let response = match server_state_lock.register(&username, &password) {
                    Ok(()) => {
                        let _ = server_state_lock.save_accounts("shared/data/accounts.json");
                        Message::RegisterOk
                    }
                    Err(e) => Message::RegisterErr(e.to_string()),
                };
                drop(server_state_lock);
                
                send(&mut *write_half, &response).await;
            }

            Message::Login { username, password } => {
                let server_state_lock = state.lock().await;

                if server_state_lock.is_online(&username) {
                    drop(server_state_lock);
                    send(&mut *write_half, &Message::LoginErr(
                        format!("Session already active for '{}'", username)
                    )).await;
                    continue;
                }

                let auth_result = server_state_lock.authenticate(&username, &password);
                drop(server_state_lock);

                match auth_result {
                    Ok(()) => {
                        send(&mut *write_half, &Message::LoginOk).await;
                        return Some(username); // Autenticazione completata
                    }
                    Err(e) => {
                        send(&mut *write_half, &Message::LoginErr(e.to_string())).await;
                    }
                }
            }

            _ => {
                send(&mut *write_half, &Message::LoginErr(
                    "You must log in first".to_string()
                )).await;
            }
        }
    }
}



// Inizializza le strutture necessarie per mantenere attiva la 
// comunicazione post-login. Crea un canale mpsc in cui il server può 
// depositare i messaggi destinati a questo utente. Avvia quindi un task 
// asincrono dedicato in background che, non appena vede arrivare un messaggio 
// in questo canale, lo spedisce al socket del client tramite la sua parte di 
// scrittura.

async fn setup_active_session(username: &str, mut write_half: OwnedWriteHalf, state: &Arc<Mutex<ServerState>>) -> JoinHandle<()> {

    let (tx, mut rx) = mpsc::channel::<Message>(32);

    {
        let mut server_state_lock = state.lock().await;
        server_state_lock.login(username, tx);
    }

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            send(&mut write_half, &msg).await;
        }
    })
}



// Mantiene aperto il canale di ascolto per tutta la durata della sessione.
// Legge riga per riga i comandi in entrata (come aggiornamenti di posizione o messaggi in chat),
// li converte in strutture dati Rust (Message) e li smista per l'elaborazione.
// Il loop si interrompe solo in caso di disconnessione o errore del client.

async fn process_client_messages(
    username: &str,
    reader: &mut Lines<BufReader<OwnedReadHalf>>,
    state: &Arc<Mutex<ServerState>>,
) {
    while let Ok(Some(line)) = reader.next_line().await {
        // Prima proviamo a interpretare la riga come Message.
        // Questa categoria comprende chat, richieste analytics, login/logout, ecc.
        if let Ok(msg) = serde_json::from_str::<Message>(&line) {
            match msg {
                Message::Text(text) => {
                    println!("[{username}] Messaggio ricevuto: {text}");
                }

                // Le richieste analytics saranno gestite nello step successivo.
                _ => {
                    println!("[{username}] Messaggio non ancora gestito: {:?}", msg);
                }
            }

            continue;
        }

        // Se non è un Message, proviamo a interpretarlo come UpdatePosition.
        // Il client invia così gli aggiornamenti di posizione letti dal file percorso.txt.
        if let Ok(update_position) = serde_json::from_str::<UpdatePosition>(&line) {
            let mut server_state_lock = state.lock().await;

            // Aggiorna lo stato dell'utente e salva la posizione nella history del TrackerState.
            server_state_lock.process_packet(update_position);

            continue;
        }

        // Se il pacchetto non è né Message né UpdatePosition, lo segnaliamo.
        eprintln!("[{username}] Pacchetto non riconosciuto: {line}");
    }
}



// Si occupa di pulire le risorse non appena l'utente chiude l'applicazione.
// Rimuove l'utente dal registro degli account attualmente online all'interno dello stato globale 
// e interrompe forzatamente il task dedicato alla scrittura dei messaggi, liberando così la memoria.

async fn disconnect_client(username: &str, state: &Arc<Mutex<ServerState>>, writer_task: JoinHandle<()>) {

    let mut server_state_lock = state.lock().await;
    server_state_lock.logout(username);
    drop(server_state_lock);
    
    writer_task.abort();

    println!("\t\t\t\tUser disconnected: {username}");
}



// Formatta un pacchetto dati per la spedizione sulla rete. Prende la struttura
// dati Message, la converte in una stringa di testo in formato JSON e vi 
// aggiunge il carattere di a capo finale necessario affinché il ricevente 
// capisca che il messaggio è concluso.

async fn send(writer: &mut (impl AsyncWriteExt + Unpin), msg: &Message) {
    let json = serde_json::to_string(msg).unwrap();
    let _ = writer.write_all(format!("{json}\n").as_bytes()).await;
}