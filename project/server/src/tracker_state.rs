use chrono::DateTime;
use serde::{Serialize, Deserialize};
use shared::coordinates::Coordinates;
use shared::update_position::UpdatePosition;
use shared::user_state::UserState;
use crate::analytics::{AnalyticsConfig, haversine_distance_km};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrackerState {

    last_coordinates: Option<Coordinates>,
    state: UserState,
    last_move: Option<DateTime<chrono::Utc>>,
    history: Vec<UpdatePosition>,
    movement_threshold: f64,
    pause_threshold_secs: i64,
    
}

impl TrackerState {
    
    
    
    // Inizializzazione.

    pub fn new() -> Self {
        let default_config = AnalyticsConfig::default();

        TrackerState {
            last_coordinates: None,
            state: UserState::Sconnesso,
            last_move: None,
            history: Vec::new(),
            movement_threshold: default_config.movement_threshold_meters,
            pause_threshold_secs: default_config.pause_threshold.as_secs() as i64
        }
    }



    // Restituisce un rapido riferimento in sola lettura all'ultima posizione registrata.

    pub fn get_coordinates(&self) -> &Option<Coordinates> {
        &self.last_coordinates
    }



    // Estrae e fornisce all'esterno l'attuale stato dinamico dell'utente.

    pub fn get_state(&self) -> &UserState {
        &self.state
    }



    // Restituisce il puntatore temporale corrispondente al momento dell'ultimo movimento.

    pub fn get_last_move(&self) -> &Option<DateTime<chrono::Utc>> {
        &self.last_move
    }

    pub fn get_history(&self) -> &Vec<UpdatePosition> {
        &self.history
    }

    pub fn set_disconnected(&mut self) {
        self.state = UserState::Sconnesso;
    }

    pub fn update_position(&mut self, new_update: &UpdatePosition) {

        let new_coordinates = &new_update.coordinates;
        let new_time = new_update.time;
        self.history.push(new_update.clone());  //aggiunto

        match &self.last_coordinates {

            None => {

                self.state = UserState::Fermo;
                self.last_move = Some(new_time);
                self.last_coordinates = Some(new_coordinates.clone());
                println!("[{}] Connesso. Stato iniziale: FERMO", new_update.username);

            }

            Some(last_coordinates) => {

                let distance_m = haversine_distance_km(
                    last_coordinates.get_latitude(),
                    last_coordinates.get_longitude(),
                    new_coordinates.get_latitude(),
                    new_coordinates.get_longitude(),
                ) * 1000.0;

                if distance_m > self.movement_threshold {
                    
                    if !matches!(&self.state, UserState::InMovimento) {

                        println!("[{}] Transizione: FERMO -> IN MOVIMENTO", new_update.username);
                        self.state = UserState::InMovimento;
                    
                    }

                    self.last_move = Some(new_time);
                    self.last_coordinates = Some(new_coordinates.clone());

                } else {

                    if matches!(&self.state, UserState::InMovimento) {

                        if let Some(last_move_time) = self.last_move {

                            let duration = new_time.signed_duration_since(last_move_time);

                            if duration.num_seconds() >= self.pause_threshold_secs {

                                println!("[{}] Transizione: IN MOVIMENTO -> FERMO (sosta di {} sec)",
                                    new_update.username, duration.num_seconds());
                                self.state = UserState::Fermo;
                                
                            }
                        }
                    }
                }
            }
        }
    }
}