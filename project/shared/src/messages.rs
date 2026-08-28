use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Message {
    Register { username: String, password: String },
    Login { username: String, password: String },

    RegisterOk,
    RegisterErr(String),
    LoginOk,
    LoginErr(String),
    
    Text(String), 

    SendDirectMessage { to: String, text: String },
    IncomingDirectMessage { from: String, text: String },
    
    SendBroadcastMessage { text: String },
    IncomingBroadcastMessage { from: String, text: String },
}