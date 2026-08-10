use serde::{Serialize, Deserialize};
use crate::coordinates::Coordinates;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdatePosition {
    
    pub username: String,
    pub coordinates: Coordinates,
    pub time: String
}