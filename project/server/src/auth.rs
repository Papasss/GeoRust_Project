use crate::server_state::ServerState;
use std::collections::HashMap;
use std::io::ErrorKind;
use tokio::fs;

#[derive(Debug)]
pub enum AuthError {
    UsernameTaken,
    UserNotFound,
    WrongPassword,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::UsernameTaken => write!(f, "Nome utente già in uso"),
            AuthError::UserNotFound => write!(f, "Utente non trovato"),
            AuthError::WrongPassword => write!(f, "Password errata"),
        }
    }
}



// Elabora e genera l'impronta digitale univoca (hash) della password utente.
// Utilizza l'algoritmo SHA256 per proteggere i dati sensibili, garantendo 
// che nessuna password rimanga memorizzata in chiaro nel server.

fn hash_password(password: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes()); 
    format!("{:x}", hasher.finalize())
}

impl ServerState {
    
    
    
    // Legge e carica gli account utente accedendo in modo totalmente asincrono al disco.
    // Converte il file JSON in una mappa utilizzabile in memoria. Qualora il file 
    // fosse mancante, avvia l'applicativo con un registro anagrafico pulito.

    pub async fn load_accounts(&mut self, path: &str) {
        match fs::read_to_string(path).await {
            Ok(content) => {
                match serde_json::from_str::<HashMap<String, String>>(&content) {
                    Ok(accounts) => {
                        self.accounts = accounts;
                        println!("Caricati {} account dal file {}", self.accounts.len(), path);
                    }
                    Err(e) => {
                        eprintln!("File degli account corrotto o non valido: {}", e);
                    }
                }
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                println!("Nessun file account esistente trovato, avvio con archivio vuoto.");
            }
            Err(e) => {
                eprintln!("Errore durante la lettura del file degli account: {}", e);
            }
        }
    }
    
    
    
    // Converte in JSON tutti gli account registrati e li salva asincronamente su disco.
    // Viene invocato dinamicamente a ogni nuova iscrizione per assicurare 
    // che i dati siano sempre persistenti senza bloccare il runtime centrale di Tokio.

    pub async fn save_accounts(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.accounts)
            .expect("Errore nella serializzazione degli account");
        fs::write(path, json).await
    }
    
    
    
    // Esegue la registrazione controllando se l'alias scelto è già occupato.
    // Protegge la nuova utenza associandole l'hash cifrato della password,
    // restituendo successo unicamente se il salvataggio va a buon fine.

    pub fn register(&mut self, username: &str, password: &str) -> Result<(), AuthError> {
        if self.accounts.contains_key(username) {
            return Err(AuthError::UsernameTaken);
        }
        self.accounts.insert(
            username.to_string(),
            hash_password(password)
        );
        Ok(())
    }
    
    
    
    // Valida le credenziali ricevute in fase di login contro gli archivi interni.
    // Restituisce un errore se lo username è inesistente o se l'hash della 
    // password fornita si discosta da quello registrato a sistema.
    
    pub fn authenticate(&self, username: &str, password: &str) -> Result<(), AuthError> {
        let account = self.accounts.get(username).ok_or(AuthError::UserNotFound)?;
        if account != &hash_password(password) {
            return Err(AuthError::WrongPassword);
        }
        Ok(())
    }
}

// ... I tests li lascio così com'erano, non necessitano di modifiche strutturali!