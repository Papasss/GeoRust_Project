//AUTENTICAZIONE E REGISTRAZIONE
use crate::serverState::ServerState;
use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
pub enum AuthError {
    UsernameTaken,
    UserNotFound,
    WrongPassword,
}
//converte gi enum in messaggi di errore
impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::UsernameTaken => write!(f, "Username is already taken"),
            AuthError::UserNotFound => write!(f, "User not found"),
            AuthError::WrongPassword => write!(f, "Incorrect password"),
        }
    }
}

fn hash_password(password: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes()); 
    format!("{:x}", hasher.finalize())
}

// secondo blocco impl per ServerState, per accounts
impl ServerState {
    /// Carica gli account da file, se esiste. Se il file non c'è
    /// (prima esecuzione), non è un errore: si parte con zero account.
    pub fn load_accounts(&mut self, path: &str) {
        match fs::read_to_string(path) {
            Ok(content) => {
                match serde_json::from_str::<HashMap<String, String>>(&content) {
                    Ok(accounts) => {
                        self.accounts= accounts;
                        println!("Loaded {} accounts from {}", self.accounts.len(), path);
                    }
                    Err(e) => {
                        eprintln!("Accounts file corrupted or malformed: {}", e);
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("No existing accounts file found, starting empty.");
            }
            Err(e) => {
                eprintln!("Error reading accounts file: {}", e);
            }
        }
    }
    /// Salva tutti gli account su file, in formato JSON
    pub fn save_accounts(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.accounts)
            .expect("Errore nella serializzazione degli account");
        fs::write(path, json)
    }

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

    pub fn authenticate(&self, username: &str, password: &str) -> Result<(), AuthError> {
        let account = self.accounts.get(username).ok_or(AuthError::UserNotFound)?;
        if account != &hash_password(password) {
            return Err(AuthError::WrongPassword);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registrazione_nuovo_utente_ha_successo() {
        let mut state = ServerState::new();
        assert!(state.register("mario", "pass123").is_ok());
    }

    #[test]
    fn registrazione_username_duplicato_fallisce() {
        let mut state = ServerState::new();
        state.register("mario", "pass123").unwrap();
        let result = state.register("mario", "altrapassword");
        assert!(matches!(result, Err(AuthError::UsernameTaken)));
    }

    #[test]
    fn login_con_credenziali_corrette_ha_successo() {
        let mut state = ServerState::new();
        state.register("mario", "pass123").unwrap();
        assert!(state.authenticate("mario", "pass123").is_ok());
    }

    #[test]
    fn login_con_password_sbagliata_fallisce() {
        let mut state = ServerState::new();
        state.register("mario", "pass123").unwrap();
        let result = state.authenticate("mario", "passsbagliata");
        assert!(matches!(result, Err(AuthError::WrongPassword)));
    }

    #[test]
    fn login_utente_inesistente_fallisce() {
        let state = ServerState::new();
        let result = state.authenticate("fantasma", "qualsiasi");
        assert!(matches!(result, Err(AuthError::UserNotFound)));
    }
}