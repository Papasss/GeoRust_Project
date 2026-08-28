use shared::read_file;
use shared::parse_values;
use shared::update_position::UpdatePosition;
use shared::coordinates::Coordinates;
use shared::messages::Message;

use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::fs;
use std::fmt::Debug;

use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncWrite;
// Serve per leggere in modo asincrono le risposte inviate dal server
// durante la sessione post-login.
use tokio::io::{AsyncBufReadExt, BufReader as TokioBufReader};
use tokio::time::Duration;
use tokio::sync::mpsc;

use serde::Serialize;
use rand::Rng;
use chrono::DateTime;

mod auth_flow;
mod menu;

//Tutto ciò che può essere spedito al server dal client
pub enum Outgoing {
    Chat(Message),
    Position(UpdatePosition),
}

pub async fn send_to_server<T, W>(writer: &mut W, pacchetto: &T)
where
    T: Serialize + Debug,
    W: AsyncWrite + Unpin,
{
    let json_data = serde_json::to_string(pacchetto).expect("Errore nella conversione in JSON");

    if let Err(e) = writer.write_all(format!("{}\n", json_data).as_bytes()).await {
        eprintln!("Errore durante l'invio dei dati al server: {}", e);
    }
}

/// Legge una riga da terminale, con un prompt, e la ripulisce
pub fn read_line_trimmed(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Errore durante la lettura dell'input");
    input.trim().to_string()
}

fn create_random_path(file_path: &str) {
    let mut file = fs::File::create(file_path).expect("Impossibile creare il file del percorso");
    let mut rng = rand::thread_rng();
    
    let mut lat_attuale = 45.0618513;
    let mut lon_attuale = 7.6606506;
    let mut tempo_secondi = 0;
    
    // Generiamo 10 posizioni per l'emulazione
    for _ in 0..10 {
        // Probabilità del 20% che l'utente stia fermo
        let sta_fermo = rng.gen_bool(0.2); 
        
        if !sta_fermo {
            // Movimento: cambiamo leggermente le coordinate
            lat_attuale += rng.gen_range(-0.005..0.005);
            lon_attuale += rng.gen_range(-0.005..0.005);
        }
        
        // Creiamo un timestamp fittizio (es. "2026-07-08T07:XX:XXZ")
        let minuti = tempo_secondi / 60;
        let secondi = tempo_secondi % 60;
        let timestamp = format!("2026-07-08T07:{:02}:{:02}Z", minuti, secondi);
        
        // Scriviamo la riga nel formato "Lat, Lon, Timestamp"
        writeln!(
            file, 
            "{},{},{}", 
            lat_attuale, lon_attuale, timestamp
        ).unwrap();
        
        // Incrementiamo di 30 secondi (il passo richiesto per l'invio)
        tempo_secondi += 30;
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("Avvio dell'applicazione Client...");

    loop {
        let mut stream = match TcpStream::connect("127.0.0.1:8080").await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Impossibile connettersi al server: {}", e);
                return Ok(());
            }
        };
        println!("Connessione al server stabilita!");


        // --- FASE DI AUTENTICAZIONE ---
        let username = auth_flow::run_auth_flow(&mut stream).await;
        
        // TEST
        //let username = "Vanessa".to_string(); 

        // --- GESTIONE DELLA CARTELLA E DEL FILE ---
        let user_dir = format!("client/{}", username);
        let file_path = format!("{}/percorso.txt", user_dir);
        let path = Path::new(&user_dir);

        // Se la cartella dell'utente non esiste, la creiamo e generiamo le coordinate
        if !path.exists() {
            println!("\nCreazione della cartella e del percorso di {} ...", username);
            fs::create_dir_all(path)?;
            create_random_path(&file_path);
        } else {
            println!("\nUtente '{}' esistente. Lettura del file esistente...", username);
        }

        // --- FASE DI LETTURA DEL FILE ---
        let mut percorso: Vec<UpdatePosition> = Vec::new();

        let file = read_file(&file_path)?;
        let reader = BufReader::new(file);

        for line_result in reader.lines() {
            let line = line_result?;
            let values = parse_values(&line);

            if values.is_empty() {
                continue;
            }

        let coordinates  = Coordinates::new(values[0].clone(), values[1].clone());
        let time = DateTime::parse_from_rfc3339(&values[2])
            .map(|time| time.with_timezone(&chrono::Utc))
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

            let update_position = UpdatePosition {
                username: username.clone(),
                coordinates,
                time
            };

            percorso.push(update_position);
        }

        if percorso.is_empty() {
            println!("Nessuna posizione trovata nel file {}", file_path);
            return Ok(());
        }
    
        // --- split + canale condiviso ---
        let (read_half, mut write_half) = stream.into_split();
        let (tx, mut rx) = mpsc::channel::<Outgoing>(32);

        // Task READER:
        // resta in ascolto delle risposte del server, ad esempio AnalyticsResponse.
        // Senza questo task il client invierebbe richieste, ma non mostrerebbe mai le risposte.
        let reader_task = tokio::spawn(async move {
            let mut reader = TokioBufReader::new(read_half).lines();

            while let Ok(Some(line)) = reader.next_line().await {
                match serde_json::from_str::<Message>(&line) {
                    Ok(Message::AnalyticsResponse(response)) => {
                        println!("\n=== Risultato analytics ===");
                        println!("{response}");
                        println!("===========================\n");
                    }

                    Ok(Message::AnalyticsErr(error)) => {
                        eprintln!("\nErrore analytics: {error}\n");
                    }

                    Ok(other_message) => {
                        println!("\nMessaggio dal server: {:?}\n", other_message);
                    }

                    Err(error) => {
                        eprintln!("Risposta non valida dal server: {error}");
                    }
                }
            }
        });

        let writer_task = tokio::spawn(async move {
            while let Some(pkt) = rx.recv().await {
                match pkt {
                    Outgoing::Chat(msg) => send_to_server(&mut write_half, &msg).await,
                    Outgoing::Position(upd) => send_to_server(&mut write_half, &upd).await,
                }
            }
        });

        // task WRITER
                
        let username_clone = username.clone();
        let tx_positions = tx.clone();
        let position_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            for upd in percorso {
                interval.tick().await;
                if tx_positions.send(Outgoing::Position(upd)).await.is_err() {
                    break;
                }
            }
            println!("\nPercorso completato per l'utente {}.", username_clone);
        });
        // --- MENU PRINCIPALE---
        let action = menu::run_main_menu(&tx, &username).await;

        match action {
           menu::MenuAction::Logout => {
                 position_task.abort();
                 reader_task.abort();
                 drop(tx);
                 let _ = writer_task.await;
                 println!("Tornando al menu iniziale...\n");
                 continue;
            }
        }
    }

    Ok(())
}