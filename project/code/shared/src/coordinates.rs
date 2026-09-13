use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Coordinates {
    latitude: f64,
    longitude: f64,
    timestamp: i64,
}

impl Coordinates {
    
    
    
    // Istanzia un nuovo oggetto Coordinates

    pub fn new(lat_str: String, lon_str: String, timestamp: i64) -> Self {

        let latitude = lat_str.parse::<f64>().unwrap_or(0.0);
        let longitude = lon_str.parse::<f64>().unwrap_or(0.0);
        Self {latitude, longitude, timestamp}
        
    }



    // Restituisce il valore numerico della latitudine attualmente memorizzata.

    pub fn get_latitude(&self) -> f64 {

        self.latitude

    }



    // Restituisce il valore numerico della longitudine attualmente memorizzata.
    
    pub fn get_longitude(&self) -> f64 {

        self.longitude

    }



    // Restituisce il timestamp della coordinata

    pub fn get_timestamp(&self) -> i64 {

        self.timestamp

    }
}