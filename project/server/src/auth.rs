use crate::server_state::ServerState;
use bcrypt::{hash, verify, DEFAULT_COST};

// Raccoglie e classifica gli errori generati durante il processo di autenticazione.
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

            AuthError::UsernameTaken => write!(f, "Username already in use"),
            AuthError::UserNotFound => write!(f, "User not found"),
            AuthError::WrongPassword => write!(f, "Incorrect password"),
            AuthError::InvalidUsername => write!(f, "Invalid username"),
            AuthError::PasswordTooShort => write!(f, "Password too short, minimum 6 characters"),
        
        }
    }
}

const MIN_USERNAME_LEN: usize = 3;
const MAX_USERNAME_LEN: usize = 20;

// Verifica la conformità sintattica bloccando path traversal, spazi e caratteri illegali.

fn is_valid_username(username: &str) -> bool {

    let len = username.chars().count();

    if len < MIN_USERNAME_LEN || len > MAX_USERNAME_LEN {

        return false;

    }

    username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}



// Elabora e genera l'hash asimmetrico della password utente.

fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).expect("Internal error while hashing password")
}



// Confronta la password fornita con l'hash crittografato registrato a sistema.

fn verify_password(password: &str, hashed: &str) -> bool {
    verify(password, hashed).unwrap_or(false)
}



impl ServerState {

    // Esegue la registrazione controllando la presenza di un duplicato.

    pub async fn register(&self, username: &str, password: &str) -> Result<(), AuthError> {
        
        if !is_valid_username(username) {
            
            return Err(AuthError::InvalidUsername);
        }

        if password.len() < 6 {
            
            return Err(AuthError::PasswordTooShort);
        
        }

        let mut accounts = self.accounts.write().await;
        
        if accounts.contains_key(username) {
            
            return Err(AuthError::UsernameTaken);
        
        }

        accounts.insert(username.to_string(), hash_password(password));
        Ok(())
    }



    // Valida le credenziali ricevute in fase di login interrogando la mappa in sola lettura.
    
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<(), AuthError> {
        
        let accounts = self.accounts.read().await;
        let account = accounts.get(username).ok_or(AuthError::UserNotFound)?;

        if !verify_password(password, account) {
            
            return Err(AuthError::WrongPassword);
        
        }
        Ok(())
    }
}