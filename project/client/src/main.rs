use shared::read_file;
use shared::parse_values;
use shared::update_position::UpdatePosition;
use shared::coordinates::Coordinates;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::fs;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio::time::Duration;
use serde::Serialize;
use std::fmt::Debug;
use rand::Rng;

mod auth_flow;

async fn send_to_server<T>(stream: &mut TcpStream, pacchetto: &T) where T: Serialize + Debug {
    let json_data = serde_json::to_string(pacchetto).expect("Errore nella conversione in JSON");

    if let Err(e) = stream.write_all(format!("{}\n", json_data).as_bytes()).await {
        eprintln!("Errore durante l'invio dei dati al server: {}", e);
    } else {
        println!("[OK] Inviato: {:?}", pacchetto);
    }
}


fn genera_percorso_random(file_path: &str) {
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
    println!("Starting Client Application...");

    let mut stream = match TcpStream::connect("127.0.0.1:8080").await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
            return Ok(());
        }
    };
    println!("Connect to server!");

    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // --- FASE DI AUTENTICAZIONE ---
    let username = auth_flow::run_auth_flow(&mut stream, &mut reader);
    
    // TEST
    //let username = "Vanessa".to_string(); 

    println!("Authenticated successfully as: {}", username);

    // --- GESTIONE DELLA CARTELLA E DEL FILE ---
    let user_dir = format!("client/{}", username);
    let file_path = format!("{}/percorso.txt", user_dir);
    let path = Path::new(&user_dir);

    // Se la cartella dell'utente non esiste, la creiamo e generiamo le coordinate
    if !path.exists() {
        println!("Nuovo utente '{}'. Creazione della cartella e del percorso...", username);
        fs::create_dir_all(path)?;
        genera_percorso_random(&file_path);
    } else {
        println!("Utente '{}' esistente. Lettura del file esistente...", username);
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
        let time = values[2].clone();

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

    // --- FASE DI INVIO COORDINATE ---
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    
    for upd in percorso {
        // Aspettiamo 30 secondi (il server riceve ogni 30 secondi la posizione di ogni utente[cite: 1])
        interval.tick().await; 
        
        send_to_server(&mut stream, &upd).await;
    }

    println!("Percorso completato per l'utente {}. Client in chiusura.", username);
    Ok(())
}