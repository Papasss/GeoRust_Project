use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum UserState {

    Sconnesso = -1, 
    Fermo = 0,
    InMovimento = 1
    
}