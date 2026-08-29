use std::{collections::HashMap, time::Duration};
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use shared::messages::Message;
use cpu_time::ProcessTime;
use shared::update_position::UpdatePosition;
use crate::tracker_state::TrackerState;

pub struct ServerState {
    pub connections: HashMap<String, mpsc::Sender<Message>>,
    pub accounts: HashMap<String, String>, 
    pub users: HashMap<String, TrackerState>,
}

impl ServerState {
    
    
    
    // Istanzia il registro principale del server.

    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            accounts: HashMap::new(),
            users: HashMap::new(),
        }
    }



    // Attiva lo stato online inserendo il client appena loggato nella mappa.

    pub fn login(&mut self, username: &str, sender: mpsc::Sender<Message>) {
        self.connections.insert(username.to_string(), sender);
    }



    // Rimuove i dati di rete attivi scollegando formalmente il client.

    pub fn logout(&mut self, username: &str) {
        self.connections.remove(username);
    }



    // Controlla rapidamente all'interno della mappa delle chiavi di connessione
    // se lo specifico utente è segnato come correntemente online nel sistema.

    pub fn is_online(&self, username: &str) -> bool {
        self.connections.contains_key(username) 
    }



    // Invia un messaggio diretto.

    pub async fn direct_message(&self, receiver: &str, msg: Message) -> Result<(), String> {
        
        if let Some(tx) = self.connections.get(receiver) {

            let res = tx.send(msg).await.map_err(|mex| mex.to_string());
           
            match res {

                Ok(_) => { 
                    println!("Messagge succesfully sended to:\t{receiver}");
                    Ok(())
                },

                Err(error) => {
                    println!("User '{receiver}' not connected:\tERROR: {error}");
                    Err(format!("Unable to reach'{receiver}'"))
                }
            }

        } else {

            Err(format!("User '{}' not connected", receiver))

        }
    }



    // Invia un messaggio broadcast

    pub async fn broadcast(&self, msg: Message) {

        for user in self.connections.keys() {

            let _ = self.direct_message(user, msg.clone()).await;

        }
    }



    // Genera un task in background che monitora periodicamente le performance
    // ogni 2 minuti

    pub async fn start_log_cpu_usage() {

        let mut timer = interval(Duration::from_secs(120));

        tokio::spawn(async move {

            loop {

                timer.tick().await;

                let cpu_duration = Self::get_cpu_time();

                if cpu_duration.is_some() {

                    let err =  Self::write_cpu_usage(cpu_duration.unwrap()).await;

                    if err.is_err() {
                        
                        eprintln!("[CPU_USAGE] Error while writing logs:\t{}", err.unwrap_err());
                    
                    }
                }
            }
        });
    }



    // Legge le statistiche del sistema operativo host recuperando
    // specificamente la durata del processo impiegata sul processore.

    fn get_cpu_time() -> Option<Duration> {
        let cpu_now = ProcessTime::now();
        Some(cpu_now.as_duration())
    }



    // Determina tramite cargo il sistema operativo corrente e definisce la 
    // directory corretta per il salvataggio delle risorse

    #[cfg(target_os = "windows")]
    fn get_log_dir() -> &'static str {
        "server/logs/windows"
    }
    #[cfg(target_os = "macos")]
    fn get_log_dir() -> &'static str {
        "server/logs/macos"
    }
    #[cfg(target_os = "linux")]
    fn get_log_dir() -> &'static str {
        "server/logs/linux"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn get_log_dir() -> &'static str {
        "server/logs/other"
    }



    // Salva l'utilizzo della CPU su file

    async fn write_cpu_usage(cpu_time: Duration) -> Result<(), std::io::Error> {

        let path = Self::get_log_dir();

        tokio::fs::create_dir_all(path).await?;

        let mut file = OpenOptions::new().create(true).append(true).open(format!("{path}/cpu_performance.log")).await?;
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_line = format!("[{}] Server's CPU time: {:.6} s\n", timestamp, cpu_time.as_secs_f64());

        file.write_all(log_line.as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }



    // Smista un nuovo pacchetto di aggiornamento coordinate inviato via socket.
    // Richiama il modulo di tracciamento o ne instanzia uno nuovo a seconda se
    // l'utente sia già associato a uno stato tracker pregresso in memoria.
    
    pub fn process_packet(&mut self, packet: UpdatePosition) {
        
        let tracker = self.users
            .entry(packet.username.clone())
            .or_insert_with(TrackerState::new);

        tracker.update_position(&packet);
    }
}