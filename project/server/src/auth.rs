use crate::server_state::ServerState;
use std::collections::HashMap;
use std::io::ErrorKind;
use tokio::fs;
use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Debug)]
pub enum AuthError {

    UsernameTaken,
    UserNotFound,
    WrongPassword,
    InvalidUsername,
    PasswordTooShort,

}

impl std::fmt::Display for AuthError {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {

            AuthError::UsernameTaken => write!(f, "Nome utente già in uso"),
            AuthError::UserNotFound => write!(f, "Utente non trovato"),
            AuthError::WrongPassword => write!(f, "Password errata"),
            AuthError::InvalidUsername => write!(f, "Nome utente non valido"),
            AuthError::PasswordTooShort => write!(f, "Password troppo corta, minimo 6 caratteri"),
        }
    }
}



// Elabora e genera l'hash della password utente.

fn hash_password(password: &str) -> String {

    hash(password, DEFAULT_COST).expect("Errore nell'hashing della password")

}

fn verify_password(password: &str, hashed: &str) -> bool {
    
    verify(password, hashed).unwrap_or(false)

}

impl ServerState {
    
    
    
    // Legge e carica gli account utente convertendo il file JSON in una mappa 
    // utilizzabile in memoria. Qualora il file fosse mancante, avvia il tutto 
    // con un registro anagrafico pulito.

    pub async fn load_accounts(&mut self) {

        match fs::read_to_string(self.accounts_file_path()).await {

            Ok(content) => {

                match serde_json::from_str::<HashMap<String, String>>(&content) {

                    Ok(accounts) => {

                        self.accounts = accounts;
                        println!("Caricati {} account dal file {}", self.accounts.len(), self.accounts_file_path().display());
                    
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
    
    
    
    // Esegue la registrazione controllando se l'alias scelto è già occupato.

    pub fn register(&mut self, username: &str, password: &str) -> Result<(), AuthError> {

        if username.trim().is_empty() {

            return Err(AuthError::InvalidUsername);
        

        }

        if password.len() < 6 {

            return Err(AuthError::PasswordTooShort);
        
        
        }

        if self.accounts.contains_key(username) {

            return Err(AuthError::UsernameTaken);

        }

        self.accounts.insert(username.to_string(), hash_password(password));
        Ok(())
    }
    
    
    
    // Valida le credenziali ricevute in fase di login.

    pub fn authenticate(&self, username: &str, password: &str) -> Result<(), AuthError> {

        let account = self.accounts.get(username).ok_or(AuthError::UserNotFound)?;

        if !verify_password(password, account) {

            return Err(AuthError::WrongPassword);

        }
        Ok(())
    }
}
