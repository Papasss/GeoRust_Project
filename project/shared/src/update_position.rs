use serde::{Serialize, Deserialize};
use crate::coordinates::Coordinates;
use chrono::DateTime;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdatePosition {
    pub username: String,
    pub coordinates: Coordinates,
    pub time: DateTime<chrono::Utc>
}