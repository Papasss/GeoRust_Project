use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use crate::server_state::ServerState;

mod server_state;
mod tracker_state;
mod auth;
mod analytics;
mod client_handler;

#[tokio::main]
async fn main() {
    println!("Starting Control Server...");

    // Stato condiviso: un solo ServerState, protetto da un solo Mutex,
    // accessibile da tutti i task client tramite Arc (possesso condiviso)
    let mut inizial_state= ServerState::new();
    let accounts_path= "../data/accounts.json";
    inizial_state.load_accounts(accounts_path);
    let state = Arc::new(Mutex::new(inizial_state));

    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to port");
    println!("Server listening on 127.0.0.1:8080");

    // Loop infinito: accetta connessioni una alla volta, e per ognuna
    // lancia un task indipendente che la gestisce
    loop {
        let (socket, addr) = listener.accept().await.unwrap(); //accetta connessione
        println!("New client connected: {addr}");

        let state = Arc::clone(&state); // clona solo il puntatore, non i dati

        tokio::spawn(async move { //lancia un task asincrono indipendente
            // La gestione completa del client è delegata al modulo client_handler,
            // che contiene anche la logica post-login per posizioni e analytics.
            client_handler::handle_client(socket, state).await;
        });
    }
}

/*
async fn handle_client(socket: TcpStream, state: Arc<Mutex<ServerState>>) {
    // Divide il socket in due metà indipendenti: una per leggere, una per scrivere
    let (read_half, mut write_half) = socket.into_split(); //permette di leggeree scrivere da due punti diversi del codice
    let mut reader = BufReader::new(read_half).lines(); //leggere i messaggi
    //permette di leggere e scrivere senza conflitti

    // FASE 1 — AUTENTICAZIONE (registrazione / login)
    let username = loop { //aspetta un login
        let line = match reader.next_line().await { //lettura riga inviata dal client
            Ok(Some(line)) => line,
            _ => {
                println!("Client disconnected during authentication phase");
                return; // esce dalla funzione, task termina
            }
        };
        //riga di testo convertita in struttura dati rust Message
        let msg: Message = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(_) => continue, // messaggio malformato, ignoro e aspetto il prossimo
        };

        match msg {
            Message::Register { username, password } => {
                let mut st = state.lock().await;
                let response = match st.register(&username, &password) {
                    Ok(()) => {
                        let _ = st.save_accounts("../shared/data/accounts.json");
                        Message::RegisterOk
                    }
                    Err(e) => Message::RegisterErr(e.to_string()),
                };
                drop(st); // rilascio il lock appena finito di usarlo
                send(&mut write_half, &response).await;
            }

            Message::Login { username, password } => {
                let st = state.lock().await;

                if st.is_online(&username) { //legge connection
                    drop(st);
                    send(&mut write_half, &Message::LoginErr(
                        "Session already active for '{username}' ".to_string()
                    )).await;
                    continue;
                }

                let auth_result = st.authenticate(&username, &password); //legge accounts
                drop(st); // rilascio il lock

                match auth_result {
                    Ok(()) => {
                        send(&mut write_half, &Message::LoginOk).await;
                        break username; // auth completata, esco dal loop
                    }
                    Err(e) => {
                        send(&mut write_half, &Message::LoginErr(e.to_string())).await;
                    }
                }
            }

            _ => {
                // qualsiasi altra richiesta prima del login viene rifiutata
                send(&mut write_half, &Message::LoginErr(
                    "You must log in first".to_string()
                )).await;
            }
        }
    };

    println!("User authenticated successfully: {username}");

    // FASE 2 — SESSIONE ATTIVA

    // Canale interno: qualsiasi altro task (es. chi manda un messaggio
    // diretto a questo utente) scriverà qui per farglielo recapitare
    let (tx, mut rx) = mpsc::channel::<Message>(32);

    {
        let mut st = state.lock().await;
        st.login(&username, tx);
    } // il lock si rilascia qui, appena finisce il blocco

    // Task dedicato: pesca dal canale e scrive davvero sul socket.
    // Gira in parallelo al loop di lettura sottostante.
    let writer_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            send(&mut write_half, &msg).await;
        }
    });

    // Loop di lettura post-login: qui arriveranno posizioni e messaggi,
    // di competenza degli altri moduli del team (per ora solo placeholder)
    while let Ok(Some(line)) = reader.next_line().await {
        let update_position: UpdatePosition = match serde_json::from_str(&line) {
            Ok(update_position) => update_position,
            Err(error) => {
                eprintln!("Invalid UpdatePosition received: {error}");
                continue;
            }
        };

        let mut st = state.lock().await;
        st.process_packet(update_position);
    }

    // FASE 3 — PULIZIA A DISCONNESSIONE

    {
        let mut st = state.lock().await;
        st.logout(&username);
    }
    writer_task.abort(); // ferma il task di scrittura, non serve più
    println!("User disconnected: {username}");
}

/// Serializza un Message in JSON e lo scrive sul socket, seguito da \n
/// (il \n serve perché BufRead::lines() legge un messaggio per riga)
async fn send(writer: &mut (impl AsyncWriteExt + Unpin), msg: &Message) {
    let json = serde_json::to_string(msg).unwrap();
    let _ = writer.write_all(format!("{json}\n").as_bytes()).await;
}
*/