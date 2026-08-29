use chrono::DateTime;
use serde::{Serialize, Deserialize};
use shared::coordinates::Coordinates;
use shared::update_position::UpdatePosition;
use shared::user_state::UserState;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrackerState {
    last_coordinates: Option<Coordinates>,
    state: UserState,
    last_move: Option<DateTime<chrono::Utc>>,
}

impl TrackerState {
    
    
    
    // Inizializzazione.

    pub fn new() -> Self {
        TrackerState { last_coordinates: None, state: UserState::Sconnesso, last_move: None }
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



    // Aggiorna la posizione.
    
    pub fn update_position(&mut self, new_update: &UpdatePosition) {

        let new_coordinates = &new_update.coordinates;
        let new_time = new_update.time;

        match &self.last_coordinates {

            None => {

                self.state = UserState::Fermo;
                self.last_move = Some(new_time);
                self.last_coordinates = Some(new_coordinates.clone());
                println!("[{}] Connesso. Stato iniziale: FERMO", new_update.username);

            }

            Some(last_coordinates) => {

                if last_coordinates.get_latitude() != new_coordinates.get_latitude() || last_coordinates.get_longitude() != new_coordinates.get_longitude() {
                    
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

                            if duration.num_seconds() >= 180 {

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