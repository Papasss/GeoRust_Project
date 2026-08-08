use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Coordinates {
    
    latitude: f32,
    longitude: f32
}