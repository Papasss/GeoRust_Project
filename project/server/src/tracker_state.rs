use chrono::DateTime;
use log::info;
use serde::{Serialize, Deserialize};
use shared::coordinates::Coordinates;
use shared::update_position::UpdatePosition;
use shared::user_state::UserState;

// Traccia lo storico e lo stato vitale (fermo/in movimento) di un singolo veicolo.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrackerState {

    last_coordinates: Option<Coordinates>,
    state: UserState,
    last_move: Option<DateTime<chrono::Utc>>,
    history: Vec<UpdatePosition>,

}



impl TrackerState {

    // Inizializza un nuovo registro di monitoraggio vuoto per il veicolo.

    pub fn new() -> Self {

        TrackerState {
            last_coordinates: None,
            state: UserState::Sconnesso,
            last_move: None,
            history: Vec::new(),
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



    // Rende accessibile l'intero storico delle coordinate memorizzate in sessione.
    pub fn get_history(&self) -> &Vec<UpdatePosition> {
        &self.history
    }



    // Segna l'utente come disconnesso interrompendo l'eventuale stato di movimento.
    pub fn set_disconnected(&mut self) {
        self.state = UserState::Sconnesso;
    }



    // Aggiorna lo stato interno basandosi sull'arrivo di nuove coordinate geografiche.
    pub fn update_position(&mut self, new_update: &UpdatePosition) {

        let new_coordinates = &new_update.coordinates;
        let new_time = new_update.time;
        self.history.push(new_update.clone());

        match &self.last_coordinates {

            None => {

                self.state = UserState::Fermo;
                self.last_move = Some(new_time);
                self.last_coordinates = Some(new_coordinates.clone());
                info!("[TRACKER]\t[{}] New tracking. Initial state assigned: STATIONARY", new_update.username);
            
            }

            Some(last_coordinates) => {

                if last_coordinates.get_latitude() != new_coordinates.get_latitude() || last_coordinates.get_longitude() != new_coordinates.get_longitude() {

                    if !matches!(&self.state, UserState::InMovimento) {

                        info!("[TRACKER]\t[{}] Movement detected. Transition: STATIONARY -> MOVING", new_update.username);
                        self.state = UserState::InMovimento;

                    }

                    self.last_move = Some(new_time);
                    self.last_coordinates = Some(new_coordinates.clone());

                } else {

                    if matches!(&self.state, UserState::InMovimento) {

                        if let Some(last_move_time) = self.last_move {

                            let duration = new_time.signed_duration_since(last_move_time);

                            if duration.num_seconds() >= 180 {

                                info!("[TRACKER]\t[{}] Prolonged stop ({} sec). Transition: MOVING -> STATIONARY", 
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