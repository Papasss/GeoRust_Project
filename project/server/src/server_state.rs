use std::{collections::HashMap, time::Duration, path::{PathBuf, Path}, io};
use log::{info, error};
use tokio::sync::{mpsc, RwLock};
use chrono::DateTime;
use tokio::time::interval;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use shared::{coordinates::Coordinates, messages::Message, parse_values};
use tokio::fs::{self, OpenOptions};
use cpu_time::ProcessTime;
use shared::update_position::UpdatePosition;
use crate::tracker_state::TrackerState;

pub struct ServerState {

    pub connections: RwLock<HashMap<String, mpsc::Sender<Message>>>,
    pub accounts: RwLock<HashMap<String, String>>,
    pub users: RwLock<HashMap<String, TrackerState>>,
    accounts_file_path: PathBuf,

}



// Crea una nuova istanza vuota dello stato del server.

impl ServerState {

    pub fn new() -> Self {

        let accounts_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data").join("accounts.json");
        Self {

            connections: RwLock::new(HashMap::new()),
            accounts: RwLock::new(HashMap::new()),
            users: RwLock::new(HashMap::new()),
            accounts_file_path,

        }
    }



    // Restituisce il percorso fisico al file JSON degli account.

    pub fn accounts_file_path(&self) -> &Path {
        &self.accounts_file_path
    }



    // Registra un utente come connesso all'interno del server.

    pub async fn try_login(&self, username: &str, sender: mpsc::Sender<Message>) -> Result<(), String> {

        let mut connections = self.connections.write().await;
        
        if connections.contains_key(username) {
            
            return Err(format!("Session already active for '{}'", username));
        }

        connections.insert(username.to_string(), sender);
        Ok(())
    }



    // Gestisce la disconnessione rimuovendo la voce utente.

    pub async fn logout(&self, username: &str) {
        self.connections.write().await.remove(username);
    }


    
    // Verifica rapidamente se un determinato utente è attualmente connesso.

    pub async fn is_online(&self, username: &str) -> bool {
        self.connections.read().await.contains_key(username)
    }



    // Legge il file JSON popolando la memoria degli account.

    pub async fn load_accounts(&self) {

        match fs::read_to_string(&self.accounts_file_path).await {

            Ok(content) => {

                match serde_json::from_str::<HashMap<String, String>>(&content) {

                    Ok(loaded) => {

                        let count = loaded.len();
                        let mut accounts = self.accounts.write().await;
                        *accounts = loaded;
                        info!("[SERVER]\tLoaded {} accounts from the file system", count);
                    
                    }

                    Err(e) => error!("[ERROR]\t\tCorrupted or invalid accounts file: {}", e),
                }
            }

            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {

                info!("[SERVER]\tNo existing accounts file found. Starting with empty registry.");
            
            }

            Err(e) => error!("[ERROR]\t\tI/O error while reading accounts: {}", e),
        }
    }



    // Salva permanentemente gli account sul disco in formato JSON.

    pub async fn save_accounts(&self) -> io::Result<()> {

        let snapshot = self.accounts.read().await.clone();
        let json = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if let Some(parent) = self.accounts_file_path.parent() {

            fs::create_dir_all(parent).await?;

        }

        fs::write(&self.accounts_file_path, json).await
    }



    // Tenta di recapitare un pacchetto dati a uno specifico utente connesso.

    pub async fn direct_message(&self, receiver: &str, msg: Message) -> Result<(), String> {

        let tx = {

            let connections = self.connections.read().await;
            connections.get(receiver).cloned()

        };

        match tx {

            Some(tx) => {

                match tx.send(msg).await {

                    Ok(_) => {

                        info!("[CHAT]\t\tDirect message processed for: '{}'", receiver);
                        Ok(())
                    }

                    Err(error) => {

                        error!("[CHAT]\t\tUnable to reach '{}': {}", receiver, error);
                        Err(format!("User '{}' unreachable", receiver))

                    }
                }
            }
            None => Err(format!("User '{}' is not currently connected", receiver)),
        }
    }



    // Invia lo stesso messaggio a tutti gli utenti attualmente online.

    pub async fn broadcast(&self, msg: Message) {

        let senders: Vec<mpsc::Sender<Message>> = {
            let connections = self.connections.read().await;
            connections.values().cloned().collect()
        };

        for tx in senders {
            let _ = tx.send(msg.clone()).await;
        }
    }



    // Avvia un task in background per la scrittura su file dei log CPU ogni 120 secondi.
    
    pub async fn start_log_cpu_usage() {

        let mut timer = interval(Duration::from_secs(120));
        let mut last_cpu_time = Self::get_cpu_time().unwrap_or(Duration::ZERO);

        tokio::spawn(async move {

            loop {

                timer.tick().await;

                if let Some(cpu_duration) = Self::get_cpu_time() {

                    let delta_cpu = cpu_duration.saturating_sub(last_cpu_time).as_secs_f64();
                    let cpu_percent = (delta_cpu / 120.0) * 100.0;

                    if let Err(err) = Self::write_cpu_usage(cpu_duration.as_secs_f64(), cpu_percent).await {
                        
                        error!("[MONITOR]\tError saving CPU logs: {}", err);
                    
                    } else {
                        
                        info!("[MONITOR]\tPerformance metrics saved to file");
                    
                    }

                    last_cpu_time = cpu_duration;

                }
            }
        });
    }



    // Interroga il sistema operativo per calcolare il tempo di CPU consumato.

    fn get_cpu_time() -> Option<Duration> {
        let cpu_now = ProcessTime::now();
        Some(cpu_now.as_duration())
    }



    // Ritorna il percorso logicamente corretto basandosi sul sistema operativo.
    
    #[cfg(target_os = "windows")]
    fn get_log_dir() -> &'static str { "logs/windows" }
    #[cfg(target_os = "macos")]
    fn get_log_dir() -> &'static str { "logs/macos" }
    #[cfg(target_os = "linux")]
    fn get_log_dir() -> &'static str { "logs/linux" }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn get_log_dir() -> &'static str { "logs/other" }



    // Scrive materialmente l'utilizzo della CPU sul log di testo.

    async fn write_cpu_usage(cpu_time: f64, cpu_percent: f64) -> Result<(), std::io::Error> {
        
        let path = Self::get_log_dir();
        let log_dir = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path);
        
        tokio::fs::create_dir_all(&log_dir).await?;

        let file_path = format!("{log_dir}/cpu_performance.log");
        let mut file = OpenOptions::new().create(true).append(true).open(file_path).await?;
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_line = format!(
            "[{}] [PID: {}] Server CPU usage: {:.6} {:.6}%\n", 
            timestamp, 
            std::process::id(), 
            cpu_time,
            cpu_percent
        );
        
        file.write_all(log_line.as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }



    // Smista un nuovo pacchetto di aggiornamento coordinate allo stato di tracciamento.

    pub async fn process_packet(&self, packet: UpdatePosition) {

        let mut users = self.users.write().await;
        let tracker = users.entry(packet.username.clone()).or_insert_with(TrackerState::new);
        tracker.update_position(&packet);

    }



    // Legge e carica lo storico delle posizioni dell'utente dal file su disco.

    pub async fn load_user_history(username: &str) -> std::io::Result<Vec<UpdatePosition>> {
        

        let base_dir = format!("{}/data/{username}", env!("CARGO_MANIFEST_DIR"));
        // tokio::fs::create_dir_all(&base_dir).await?; Capire se toglierla
        let file_path = format!("{}/route.txt", base_dir);

        let file = match tokio::fs::File::open(file_path).await {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };

        let mut history = Vec::new();
        let mut lines = BufReader::new(file).lines();

        while let Some(line) = lines.next_line().await? {
            
            let values = parse_values(&line);
            
            if values.len() < 3 {
                continue;
            }

            let time = DateTime::parse_from_rfc3339(&values[2])
                .map(|time| time.with_timezone(&chrono::Utc))
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;

            history.push(UpdatePosition {
                username: username.to_string(),
                coordinates: Coordinates::new(values[0].clone(), values[1].clone(), time.timestamp()),
                time,
            });
        }

        Ok(history)
    }



    // Popola lo stato vitale in RAM simulando la ri-esecuzione dello storico posizioni.

    pub async fn restore_user_history(&self, username: &str, history: Vec<UpdatePosition>) {
        
        let contains = self.users.read().await.contains_key(username);

        if contains {
            return;
        }

        for position in history {
            self.process_packet(position).await;
        }
    }



    // Salva un singolo pacchetto posizione sul disco appendendolo al log storico.

    pub async fn save_position(username: &str, packet: &UpdatePosition) -> std::io::Result<()> {
        

        let base_dir = format!("{}/data/{username}", env!("CARGO_MANIFEST_DIR"));
        tokio::fs::create_dir_all(&base_dir).await?;
        let file_path = format!("{}/route.txt", base_dir);
    
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .await?;

        let row = format!(
            "{},{},{}\n",
            packet.coordinates.get_latitude(),
            packet.coordinates.get_longitude(),
            packet.time.to_rfc3339()
        );

        file.write_all(row.as_bytes()).await?;
        file.flush().await
    }
}