use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Message {

    // Per chi fa la parte di login, possibilità di specificare ulteriori tipi 
    // di messaggio come Login(Username, password) o registrazione? Per ora ho 
    // fatto solo un prototipo generico. Potremmo specificare anche i messaggi
    // con le coordinate identificando un formato che ci è comodo per la
    // gestione interna.
    
    Text(String), 
}