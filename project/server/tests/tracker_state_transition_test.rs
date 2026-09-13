#[path = "../src/analytics.rs"]
mod analytics;
#[path = "../src/tracker_state.rs"]
mod tracker_state;

use chrono::DateTime;
use tracker_state::TrackerState;
use shared::coordinates::Coordinates;
use shared::update_position::UpdatePosition;
use shared::user_state::UserState;

fn build_update(username: &str, latitude: f64, longitude: f64, seconds: i64) -> UpdatePosition {
    UpdatePosition {
        username: username.to_string(),
        coordinates: Coordinates::new(latitude.to_string(), longitude.to_string(), seconds),
        time: DateTime::from_timestamp(seconds, 0).unwrap(),
    }
}

#[test]
fn moves_to_stopped_after_three_minutes_of_same_position() {
    let mut tracker = TrackerState::new();

    tracker.update_position(&build_update("alice", 45.0, 7.0, 0));
    assert_eq!(*tracker.get_state(), UserState::Fermo);

    tracker.update_position(&build_update("alice", 45.001, 7.001, 30));
    assert_eq!(*tracker.get_state(), UserState::InMovimento);

    tracker.update_position(&build_update("alice", 45.001, 7.001, 210));
    assert_eq!(*tracker.get_state(), UserState::Fermo);
}
