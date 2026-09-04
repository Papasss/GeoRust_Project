#[cfg(test)]
mod tests {
    use super::*;
    use crate::server_state::ServerState;
    use serial_test::serial;

    fn setup_state() -> ServerState {
        ServerState::new()
    }

    //Registrazione

    #[test]
    fn test_register_new_user_succeeds() {
        let mut state = setup_state();
        let result = state.register("alice", "password123");
        assert!(result.is_ok());
        assert!(state.accounts.contains_key("alice"));
    }

    #[test]
    fn test_register_duplicate_username_fails() {
        let mut state = setup_state();
        state.register("alice", "password123").unwrap();

        let result = state.register("alice", "un_altra_password");
        assert!(matches!(result, Err(AuthError::UsernameTaken)));
    }

    #[test]
    fn test_password_is_hashed_not_stored_in_plaintext() {
        let mut state = setup_state();
        state.register("bob", "secret1").unwrap(); // "secret" da sola è 6 caratteri: al limite, meglio marginare
        // nota: "secret" ha esattamente 6 caratteri quindi passerebbe comunque,
        // ma uso "secret1" per stare sopra il limite senza ambiguità

        let stored = state.accounts.get("bob").unwrap();
        assert_ne!(stored, "secret1");
        assert!(stored.starts_with("$2b$")); // formato tipico di bcrypt
    }

    #[test]
    fn test_same_password_produces_different_hashes_with_salt() {
        let mut state = setup_state();
        state.register("user1", "samepw123").unwrap();
        state.register("user2", "samepw123").unwrap();

        let h1 = state.accounts.get("user1").unwrap();
        let h2 = state.accounts.get("user2").unwrap();

        // grazie al salt, hash diversi anche con la stessa password
        assert_ne!(h1, h2);

        // ma l'autenticazione deve comunque funzionare per entrambi
        assert!(state.authenticate("user1", "samepw123").is_ok());
        assert!(state.authenticate("user2", "samepw123").is_ok());
    }

    //Login / autenticazione

    #[test]
    fn test_authenticate_correct_credentials_succeeds() {
        let mut state = setup_state();
        state.register("carol", "mypassword").unwrap();

        let result = state.authenticate("carol", "mypassword");
        assert!(result.is_ok());
    }

    #[test]
    fn test_authenticate_wrong_password_fails() {
        let mut state = setup_state();
        state.register("dave", "correctpw").unwrap();

        let result = state.authenticate("dave", "wrongpw");
        assert!(matches!(result, Err(AuthError::WrongPassword)));
    }

    #[test]
    fn test_authenticate_nonexistent_user_fails() {
        let state = setup_state();
        let result = state.authenticate("ghost", "whatever");
        assert!(matches!(result, Err(AuthError::UserNotFound)));
    }

    //  Messaggi di errore

    #[test]
    fn test_error_display_messages() {
        assert_eq!(AuthError::UsernameTaken.to_string(), "Nome utente già in uso");
        assert_eq!(AuthError::UserNotFound.to_string(), "Utente non trovato");
        assert_eq!(AuthError::WrongPassword.to_string(), "Password errata");
    }

    #[test]
    fn test_error_display_messages_for_validation() {
        assert_eq!(AuthError::InvalidUsername.to_string(), "Nome utente non valido");
        assert_eq!(
            AuthError::PasswordTooShort.to_string(),
            "Password troppo corta, minimo 6 caratteri"
        );
    }

    //Validazione registrazione 

    #[test]
    fn test_register_empty_username_fails() {
        let mut state = setup_state();
        let result = state.register("", "password123");
        assert!(matches!(result, Err(AuthError::InvalidUsername)));
    }

    #[test]
    fn test_register_whitespace_only_username_fails() {
        let mut state = setup_state();
        let result = state.register("   ", "password123");
        assert!(matches!(result, Err(AuthError::InvalidUsername)));
    }

    #[test]
    fn test_register_short_password_fails() {
        let mut state = setup_state();
        let result = state.register("mario", "123");
        assert!(matches!(result, Err(AuthError::PasswordTooShort)));
    }

    #[test]
    fn test_register_minimum_length_password_succeeds() {
        let mut state = setup_state();
        let result = state.register("mario", "123456"); // esattamente 6 caratteri
        assert!(result.is_ok());
    }

    //Salvataggio / caricamento su file 

    struct AccountsFileGuard {
        path: std::path::PathBuf,
        original_content: Option<String>,
    }

    impl AccountsFileGuard {
        fn new(state: &ServerState) -> Self {
            let path = state.accounts_file_path().to_path_buf();
            let original_content = std::fs::read_to_string(&path).ok();
            Self { path, original_content }
        }
    }

    impl Drop for AccountsFileGuard {
        fn drop(&mut self) {
            match &self.original_content {
                Some(content) => {
                    let _ = std::fs::write(&self.path, content);
                }
                None => {
                    let _ = std::fs::remove_file(&self.path);
                }
            }
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_save_and_load_accounts_roundtrip() {
        let mut state = setup_state();
        let _guard = AccountsFileGuard::new(&state); // ripristina il file reale a fine test

        state.register("eve", "pw12345").unwrap();
        state.register("frank", "pw45678").unwrap();
        state.save_accounts().await.unwrap();

        let mut new_state = ServerState::new();
        new_state.load_accounts().await;

        assert_eq!(new_state.accounts.len(), 2);
        assert!(new_state.authenticate("eve", "pw12345").is_ok());
        assert!(new_state.authenticate("frank", "pw45678").is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_load_accounts_missing_file_starts_empty() {
        let state = setup_state();
        let _guard = AccountsFileGuard::new(&state);

        // rimuoviamo esplicitamente il file, se esiste, per simulare "assente"
        let _ = std::fs::remove_file(state.accounts_file_path());

        let mut fresh_state = ServerState::new();
        fresh_state.load_accounts().await;

        assert!(fresh_state.accounts.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_load_accounts_corrupted_file_keeps_previous_state() {
        let state = setup_state();
        let _guard = AccountsFileGuard::new(&state);

        // scrive contenuto corrotto nel percorso reale (verrà ripristinato dal guard)
        std::fs::write(state.accounts_file_path(), "questo non è json valido {{{").unwrap();

        let mut loader_state = ServerState::new();
        loader_state.register("preesistente", "password123").unwrap();
        loader_state.load_accounts().await;

        // il file è corrotto: lo stato precedente in memoria deve restare intatto
        assert!(loader_state.accounts.contains_key("preesistente"));
    }

    //Stato online/offline (login/logout su ServerState) 

    #[tokio::test]
    async fn test_login_marks_user_as_online() {
        let mut state = setup_state();
        let (tx, _rx) = tokio::sync::mpsc::channel(8);

        state.login("grace", tx);
        assert!(state.is_online("grace"));
    }

    #[tokio::test]
    async fn test_logout_marks_user_as_offline() {
        let mut state = setup_state();
        let (tx, _rx) = tokio::sync::mpsc::channel(8);

        state.login("henry", tx);
        state.logout("henry");
        assert!(!state.is_online("henry"));
    }

    #[tokio::test]
    async fn test_is_online_false_for_unknown_user() {
        let state = setup_state();
        assert!(!state.is_online("nessuno"));
    }
}