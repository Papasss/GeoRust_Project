
#[path = "../src/analytics.rs"]
mod analytics;
#[path = "../src/tracker_state.rs"]
mod tracker_state;
#[path = "../src/server_state.rs"]
mod server_state;
#[path = "../src/auth.rs"]
mod auth;

#[cfg(test)]
mod tests {
    use super::auth::AuthError;
    use super::server_state::ServerState;
    use serial_test::serial;

    fn setup_state() -> ServerState {
        ServerState::new()
    }

    //Registrazione

    #[tokio::test]
    async fn test_register_new_user_succeeds() {
        let state = setup_state();
        let result = state.register("alice", "password123").await;
        assert!(result.is_ok());
        assert!(state.accounts.read().await.contains_key("alice"));
    }

    #[tokio::test]
    async fn test_register_duplicate_username_fails() {
        let state = setup_state();
        state.register("alice", "password123").await.unwrap();

        let result = state.register("alice", "un_altra_password").await;
        assert!(matches!(result, Err(AuthError::UsernameTaken)));
    }

    #[tokio::test]
    async fn test_password_is_hashed_not_stored_in_plaintext() {
        let state = setup_state();
        state.register("bob", "secret1").await.unwrap(); // "secret" da sola è 6 caratteri: al limite, meglio marginare
        // nota: "secret" ha esattamente 6 caratteri quindi passerebbe comunque,
        // ma uso "secret1" per stare sopra il limite senza ambiguità

        let accounts = state.accounts.read().await;
        let stored = accounts.get("bob").unwrap();
        assert_ne!(stored, "secret1");
        assert!(stored.starts_with("$2b$")); // formato tipico di bcrypt
    }

    #[tokio::test]
    async fn test_same_password_produces_different_hashes_with_salt() {
        let state = setup_state();
        state.register("user1", "samepw123").await.unwrap();
        state.register("user2", "samepw123").await.unwrap();

        let accounts = state.accounts.read().await;
        let h1 = accounts.get("user1").unwrap();
        let h2 = accounts.get("user2").unwrap();

        // grazie al salt, hash diversi anche con la stessa password
        assert_ne!(h1, h2);

        // ma l'autenticazione deve comunque funzionare per entrambi
        assert!(state.authenticate("user1", "samepw123").await.is_ok());
        assert!(state.authenticate("user2", "samepw123").await.is_ok());
    }

    //Validazione username: lunghezza 

    #[tokio::test]
    async fn test_register_username_too_short_fails() {
        let state = setup_state();
        let result = state.register("ab", "password123").await; // 2 caratteri, sotto il minimo di 3
        assert!(matches!(result, Err(AuthError::InvalidUsername)));
    }

    #[tokio::test]
    async fn test_register_username_minimum_length_succeeds() {
        let state = setup_state();
        let result = state.register("abc", "password123").await; // esattamente 3 caratteri
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_username_maximum_length_succeeds() {
        let state = setup_state();
        let username = "a".repeat(20); // esattamente 20 caratteri
        let result = state.register(&username, "password123").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_username_too_long_fails() {
        let state = setup_state();
        let username = "a".repeat(21); // 21 caratteri, sopra il massimo
        let result = state.register(&username, "password123").await;
        assert!(matches!(result, Err(AuthError::InvalidUsername)));
    }

    //Validazione username: caratteri ammessi 

    #[tokio::test]
    async fn test_register_username_with_special_characters_fails() {
        let state = setup_state();
        let result = state.register("alice!", "password123").await;
        assert!(matches!(result, Err(AuthError::InvalidUsername)));
    }

    #[tokio::test]
    async fn test_register_username_with_spaces_fails() {
        let state = setup_state();
        let result = state.register("al ice", "password123").await;
        assert!(matches!(result, Err(AuthError::InvalidUsername)));
    }

    #[tokio::test]
    async fn test_register_username_with_underscore_and_hyphen_succeeds() {
        let state = setup_state();
        let result = state.register("al_ice-99", "password123").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_username_with_accented_characters_fails() {
        let state = setup_state();
        let result = state.register("élena", "password123").await;
        assert!(matches!(result, Err(AuthError::InvalidUsername)));
    }

    // Comportamento case-sensitive (documentativo, non un bug) 

    #[tokio::test]
    async fn test_register_username_is_case_sensitive() {
        let state = setup_state();
        state.register("Alice", "password123").await.unwrap();
        let result = state.register("alice", "password456").await;
        assert!(result.is_ok()); // sono considerati utenti distinti
    }

    //Login / autenticazione

    #[tokio::test]
    async fn test_authenticate_correct_credentials_succeeds() {
        let state = setup_state();
        state.register("carol", "mypassword").await.unwrap();

        let result = state.authenticate("carol", "mypassword").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_authenticate_wrong_password_fails() {
        let state = setup_state();
        state.register("dave", "correctpw").await.unwrap();

        let result = state.authenticate("dave", "wrongpw").await;
        assert!(matches!(result, Err(AuthError::WrongPassword)));
    }

    #[tokio::test]
    async fn test_authenticate_nonexistent_user_fails() {
        let state = setup_state();
        let result = state.authenticate("ghost", "whatever").await;
        assert!(matches!(result, Err(AuthError::UserNotFound)));
    }

    //  Messaggi di errore

    #[test]
    fn test_error_display_messages() {
        assert_eq!(AuthError::UsernameTaken.to_string(), "Username already in use");
        assert_eq!(AuthError::UserNotFound.to_string(), "User not found");
        assert_eq!(AuthError::WrongPassword.to_string(), "Incorrect password");
    }

    #[test]
    fn test_error_display_messages_for_validation() {
        assert_eq!(AuthError::InvalidUsername.to_string(), "Invalid username");
        assert_eq!(
            AuthError::PasswordTooShort.to_string(),
            "Password too short, minimum 6 characters"
        );
    }

    //Validazione registrazione 

    #[tokio::test]
    async fn test_register_empty_username_fails() {
        let state = setup_state();
        let result = state.register("", "password123").await;
        assert!(matches!(result, Err(AuthError::InvalidUsername)));
    }

    #[tokio::test]
    async fn test_register_whitespace_only_username_fails() {
        let state = setup_state();
        let result = state.register("   ", "password123").await;
        assert!(matches!(result, Err(AuthError::InvalidUsername)));
    }

    #[tokio::test]
    async fn test_register_short_password_fails() {
        let state = setup_state();
        let result = state.register("mario", "123").await;
        assert!(matches!(result, Err(AuthError::PasswordTooShort)));
    }

    #[tokio::test]
    async fn test_register_minimum_length_password_succeeds() {
        let state = setup_state();
        let result = state.register("mario", "123456").await; // esattamente 6 caratteri
        assert!(result.is_ok());
    }

    //Salvataggio / caricamento su file 

    struct AccountsFileGuard {
        path: std::path::PathBuf,
        original_content: Option<String>,
    }

    fn accounts_file_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("accounts.json")
    }

    impl AccountsFileGuard {
        fn new() -> Self {
            let path = accounts_file_path();
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
        let state = setup_state();
        let _guard = AccountsFileGuard::new(); // ripristina il file reale a fine test

        state.register("eve", "pw12345").await.unwrap();
        state.register("frank", "pw45678").await.unwrap();
        state.save_accounts().await.unwrap();

        let new_state = ServerState::new();
        new_state.load_accounts().await;

        assert_eq!(new_state.accounts.read().await.len(), 2);
        assert!(new_state.authenticate("eve", "pw12345").await.is_ok());
        assert!(new_state.authenticate("frank", "pw45678").await.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_load_accounts_missing_file_starts_empty() {
        let _guard = AccountsFileGuard::new();

        // rimuoviamo esplicitamente il file, se esiste, per simulare "assente"
        let _ = std::fs::remove_file(accounts_file_path());

        let fresh_state = ServerState::new();
        fresh_state.load_accounts().await;

        assert!(fresh_state.accounts.read().await.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_load_accounts_corrupted_file_keeps_previous_state() {
        let _guard = AccountsFileGuard::new();

        // scrive contenuto corrotto nel percorso reale (verrà ripristinato dal guard)
        std::fs::write(accounts_file_path(), "questo non è json valido {{{").unwrap();

        let loader_state = ServerState::new();
        loader_state.register("preesistente", "password123").await.unwrap();
        loader_state.load_accounts().await;

        // il file è corrotto: lo stato precedente in memoria deve restare intatto
        assert!(loader_state.accounts.read().await.contains_key("preesistente"));
    }

    //Stato online/offline (login/logout su ServerState) 
    #[tokio::test]
    async fn test_try_login_marks_user_as_online() {
        let state = setup_state();
        let (tx, _rx) = tokio::sync::mpsc::channel(8);

        let result = state.try_login("grace", tx).await;
        assert!(result.is_ok());
        assert!(state.connections.read().await.contains_key("grace"));
    }

    #[tokio::test]
    async fn test_logout_marks_user_as_offline() {
        let state = setup_state();
        let (tx, _rx) = tokio::sync::mpsc::channel(8);

        state.try_login("henry", tx).await.unwrap();
        state.logout("henry").await;
        assert!(!state.connections.read().await.contains_key("henry"));
    }

    #[tokio::test]
    async fn test_unknown_user_not_in_connections() {
        let state = setup_state();
        assert!(!state.connections.read().await.contains_key("nessuno"));
    }

    #[tokio::test]
    async fn test_try_login_twice_same_user_fails() {
        let state = setup_state();
        let (tx1, _rx1) = tokio::sync::mpsc::channel(8);
        let (tx2, _rx2) = tokio::sync::mpsc::channel(8);

        let first = state.try_login("ivan", tx1).await;
        assert!(first.is_ok());

        let second = state.try_login("ivan", tx2).await;
        assert!(second.is_err());

        assert!(state.connections.read().await.contains_key("ivan"));
    }

    #[tokio::test]
    async fn test_try_login_after_logout_succeeds() {
        let state = setup_state();
        let (tx1, _rx1) = tokio::sync::mpsc::channel(8);
        let (tx2, _rx2) = tokio::sync::mpsc::channel(8);

        state.try_login("julia", tx1).await.unwrap();
        state.logout("julia").await;

        let result = state.try_login("julia", tx2).await;
        assert!(result.is_ok());
    }
}