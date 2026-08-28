use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Coordinates {
    
    latitude: f32,
    longitude: f32
}

impl Coordinates {
    pub fn new(lat_str: String, lon_str: String) -> Self {
        let latitude = lat_str.parse::<f32>().unwrap_or(0.0);
        let longitude = lon_str.parse::<f32>().unwrap_or(0.0);
        Self { latitude, longitude }
    }

    pub fn get_latitude(&self) -> f32 {
        self.latitude
    }

    pub fn get_longitude(&self) -> f32 {
        self.longitude
    }
}