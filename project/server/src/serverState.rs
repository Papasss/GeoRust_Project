use std::collections::HashMap;
use tokio::sync::mpsc;
use shared::messages::Message;

pub struct ServerState {

    // String: username -> Key
    // Sender: mpsc -> Value
    //registro delle connessioni attive in tempo reale
    pub connections: HashMap<String, mpsc::Sender<Message>>,
    //campo popolato dai metodi di auth.rs-> info utenti
    //username, (username, password)
    pub accounts: HashMap<String, String>, //struttura dati che tiene tutti gli account registrati HASHMAP IN RAM, se spengo il server i dati spariscono
}

impl ServerState {
    
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            accounts: HashMap::new(),
        }
    }
    //gestione sessioni online
    pub fn login(&mut self, username: &str, sender: mpsc::Sender<Message>) {
        self.connections.insert(username.to_string(), sender);
    }
    //controllo chiusura connessione canale
    pub fn logout(&mut self, username: &str) {
        self.connections.remove(username);
    }

    pub fn is_online(&self, username: &str) -> bool {
        self.connections.contains_key(username) //essere online= chiave username presente
    }
    //invia messaggio a singolo guidatore
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


    pub async fn broadcast(&self, msg: Message) {

        for user in self.connections.keys() {
            let _ = self.direct_message(user, msg.clone()).await;
        }
    }
}