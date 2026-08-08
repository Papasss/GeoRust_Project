use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UserState {
    
    sconnesso = -1, 
    fermo = 0,
    in_movimento = 1
}