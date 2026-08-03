use std::collections::HashMap;
use tokio::sync::mpsc;
use shared::messages::Message;

pub struct ServerState {

    // String: username -> Key
    // Sender: mpsc -> Value

    pub connections: HashMap<String, mpsc::Sender<Message>>,
}

impl ServerState {
    
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

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