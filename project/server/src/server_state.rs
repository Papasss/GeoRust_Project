use std::{collections::HashMap, time::Duration};
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use shared::messages::Message;
use cpu_time::ProcessTime;
use shared::update_position::UpdatePosition;
use crate::tracker_state::TrackerState;


// Rappresenta la memoria centrale del server in esecuzione.
// Contiene un registro delle connessioni attive e una mappa con le 
// informazioni e credenziali degli account registrati.

pub struct ServerState {
    pub connections: HashMap<String, mpsc::Sender<Message>>,
    pub users: HashMap<String, TrackerState>,
}

impl ServerState {
    
    // Crea una nuova istanza vuota dello stato del server.

    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            users: HashMap::new(),
        }
    }



    // Registra un utente come "online" all'interno del server.
    // Associa lo username dell'utente al canale di comunicazione appena creato,
    // permettendo così al server di recapitargli i messaggi.

    pub fn login(&mut self, username: &str, sender: mpsc::Sender<Message>) {
        self.connections.insert(username.to_string(), sender);
    }



    // Gestisce la disconnessione di un utente.
    // Rimuove la sua voce dalla mappa delle connessioni attive.

    pub fn logout(&mut self, username: &str) {
        self.connections.remove(username);
    }



    // Verifica rapidamente se un determinato utente è attualmente connesso.

    pub fn is_online(&self, username: &str) -> bool {
        self.connections.contains_key(username)
    }



    // Tenta di recapitare un messaggio a uno specifico utente connesso.
    // Se l'utente si trova nel registro, utilizza il suo canale per inviargli 
    // il messaggio.
    // In caso di successo o errore stampa un errore a terminale.

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



    // Invia lo stesso identico messaggio a tutti gli utenti attualmente online.

    pub async fn broadcast(&self, msg: Message) {

        for user in self.connections.keys() {
            let _ = self.direct_message(user, msg.clone()).await;
        }
    }



    // Avvia un task in background dedicato al monitoraggio delle prestazioni.
    // Utilizza un timer per svegliarsi ogni due minuti e registrare il tempo di CPU 
    // consumato dall'applicazione.

    pub async fn start_log_cpu_usage() {

        let mut timer = interval(Duration::from_secs(120));

        tokio::spawn(async move {
            loop {
                
                timer.tick().await;
                let cpu_duration = Self::get_cpu_time();

                if  cpu_duration.is_some() {

                    let err =  Self::write_cpu_usage(cpu_duration.unwrap()).await;

                    if err.is_err() {
                        eprintln!("[CPU_USAGE] Error while writing logs:\t{}", err.unwrap_err());
                    }
                }
            }
        });

    }



    // Interroga direttamente il sistema operativo restituendo quanto tempo di 
    // CPU è stato effettivamente utilizzato dal processo del server da quando 
    // è stato avviato fino a questo preciso istante.

    fn get_cpu_time() -> Option<Duration> {

        let cpu_now = ProcessTime::now();
        Some(cpu_now.as_duration())

    }



    // Definisce il percorso dei log a seconda del sistema opeativo

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


    // Si occupa materialmente di formattare e scrivere i log sul disco.
    // Assicura che la cartella dei log esista, apre o crea il file di testo, 
    // e vi aggiunge una nuova riga contenente la data, l'ora e i secondi di 
    // CPU consumati.
    
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


    pub fn process_packet(&mut self, packet: UpdatePosition) {

        let tracker = self.users
            .entry(packet.username.clone())
            .or_insert_with(TrackerState::new);

        tracker.update_position(&packet);
    }

}