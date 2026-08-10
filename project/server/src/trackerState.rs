use serde::{Serialize, Deserialize};
use shared::coordinates::Coordinates;
use shared::userstate::UserState;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrackerState {
    
    coordinates: Coordinates,
    state: UserState,
    lastMove: Timestamp
}