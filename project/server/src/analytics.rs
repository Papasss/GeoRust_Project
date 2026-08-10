use chrono::{Datelike, Local, TimeZone};
use std::time::Duration;

/// Una singola posizione registrata dal server.
///
/// Per ora usiamo una struttura autonoma, così il modulo analytics
/// può essere testato anche se il resto del server non è ancora completo.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionSample {
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: u64,
}

/// Periodo temporale su cui calcolare le statistiche.
#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisPeriod {
    CurrentDay,
    CurrentWeek,
    CurrentMonth,
    Custom {
        start_timestamp: u64,
        end_timestamp: u64,
    },
}

/// Configurazione dell'analisi.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalyticsConfig {
    /// Sotto questa distanza consideriamo il veicolo fermo.
    pub movement_threshold_meters: f64,

    /// Durata minima perché una sosta venga contata come pausa.
    /// Nel progetto finale deve essere 180 secondi.
    pub pause_threshold: Duration,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            movement_threshold_meters: 1.0,
            pause_threshold: Duration::from_secs(180),
        }
    }
}

/// Risultato dell'analisi del movimento.
#[derive(Debug, Clone, PartialEq)]
pub struct MovementStatistics {
    /// Tragitto percorso: sequenza delle posizioni considerate.
    pub path: Vec<PositionSample>,

    /// Distanza totale percorsa in chilometri.
    pub total_distance_km: f64,

    /// Durata complessiva del movimento.
    pub movement_duration: Duration,

    /// Durata complessiva delle pause.
    pub pause_duration: Duration,

    /// Velocità media in km/h.
    pub average_speed_kmh: f64,
}

/// Funzione principale della tua parte.
///
/// Prende la cronologia delle posizioni di un utente,
/// seleziona solo quelle nell'intervallo richiesto
/// e calcola tragitto, distanza, movimento, pause e velocità media.
pub fn analyze_movement(
    history: &[PositionSample],
    period: AnalysisPeriod,
    config: AnalyticsConfig,
) -> MovementStatistics {
    let (start_timestamp, end_timestamp) = resolve_period(period);

    let mut samples: Vec<PositionSample> = history
        .iter()
        .filter(|sample| sample.timestamp >= start_timestamp && sample.timestamp <= end_timestamp)
        .cloned()
        .collect();

    samples.sort_by_key(|sample| sample.timestamp);

    if samples.len() < 2 {
        return MovementStatistics {
            path: samples,
            total_distance_km: 0.0,
            movement_duration: Duration::from_secs(0),
            pause_duration: Duration::from_secs(0),
            average_speed_kmh: 0.0,
        };
    }

    let mut total_distance_km = 0.0;
    let mut movement_duration = Duration::from_secs(0);
    let mut pause_duration = Duration::from_secs(0);

    let mut current_stationary_duration = Duration::from_secs(0);

    for pair in samples.windows(2) {
        let previous = &pair[0];
        let current = &pair[1];

        let delta_seconds = current.timestamp.saturating_sub(previous.timestamp);
        let delta_time = Duration::from_secs(delta_seconds);

        let distance_km = haversine_distance_km(
            previous.latitude,
            previous.longitude,
            current.latitude,
            current.longitude,
        );

        let distance_meters = distance_km * 1000.0;

        if distance_meters > config.movement_threshold_meters {
            total_distance_km += distance_km;
            movement_duration += delta_time;

            if current_stationary_duration >= config.pause_threshold {
                pause_duration += current_stationary_duration;
            }

            current_stationary_duration = Duration::from_secs(0);
        } else {
            current_stationary_duration += delta_time;
        }
    }

    if current_stationary_duration >= config.pause_threshold {
        pause_duration += current_stationary_duration;
    }

    let movement_hours = movement_duration.as_secs_f64() / 3600.0;

    let average_speed_kmh = if movement_hours > 0.0 {
        total_distance_km / movement_hours
    } else {
        0.0
    };

    MovementStatistics {
        path: samples,
        total_distance_km,
        movement_duration,
        pause_duration,
        average_speed_kmh,
    }
}

/// Converte CurrentDay / CurrentWeek / CurrentMonth
/// in un intervallo temporale concreto.
fn resolve_period(period: AnalysisPeriod) -> (u64, u64) {
    match period {
        AnalysisPeriod::Custom {
            start_timestamp,
            end_timestamp,
        } => (start_timestamp, end_timestamp),

        AnalysisPeriod::CurrentDay => {
            let now = Local::now();

            let start = Local
                .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
                .single()
                .expect("Data locale non valida");

            (start.timestamp() as u64, now.timestamp() as u64)
        }

        AnalysisPeriod::CurrentWeek => {
            let now = Local::now();

            let days_from_monday = now.weekday().num_days_from_monday() as i64;
            let monday = now.date_naive() - chrono::Duration::days(days_from_monday);

            let start = Local
                .with_ymd_and_hms(monday.year(), monday.month(), monday.day(), 0, 0, 0)
                .single()
                .expect("Data locale non valida");

            (start.timestamp() as u64, now.timestamp() as u64)
        }

        AnalysisPeriod::CurrentMonth => {
            let now = Local::now();

            let start = Local
                .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
                .single()
                .expect("Data locale non valida");

            (start.timestamp() as u64, now.timestamp() as u64)
        }
    }
}

/// Calcola la distanza in chilometri tra due coordinate geografiche.
/// Usa la formula di Haversine.
pub fn haversine_distance_km(
    lat1: f64,
    lon1: f64,
    lat2: f64,
    lon2: f64,
) -> f64 {
    let earth_radius_km = 6371.0_f64;

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();

    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let h = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);

    let c = 2.0 * h.sqrt().atan2((1.0 - h).sqrt());

    earth_radius_km * c
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(timestamp: u64, latitude: f64, longitude: f64) -> PositionSample {
        PositionSample {
            latitude,
            longitude,
            timestamp,
        }
    }

    fn test_config() -> AnalyticsConfig {
        AnalyticsConfig {
            movement_threshold_meters: 1.0,
            pause_threshold: Duration::from_secs(180),
        }
    }

    #[test]
    fn empty_history_returns_zero_statistics() {
        let stats = analyze_movement(
            &[],
            AnalysisPeriod::Custom {
                start_timestamp: 0,
                end_timestamp: 1000,
            },
            test_config(),
        );

        assert_eq!(stats.path.len(), 0);
        assert_eq!(stats.total_distance_km, 0.0);
        assert_eq!(stats.movement_duration.as_secs(), 0);
        assert_eq!(stats.pause_duration.as_secs(), 0);
        assert_eq!(stats.average_speed_kmh, 0.0);
    }

    #[test]
    fn single_position_returns_zero_statistics() {
        let history = vec![sample(0, 45.0, 7.0)];

        let stats = analyze_movement(
            &history,
            AnalysisPeriod::Custom {
                start_timestamp: 0,
                end_timestamp: 1000,
            },
            test_config(),
        );

        assert_eq!(stats.path.len(), 1);
        assert_eq!(stats.total_distance_km, 0.0);
        assert_eq!(stats.movement_duration.as_secs(), 0);
        assert_eq!(stats.pause_duration.as_secs(), 0);
        assert_eq!(stats.average_speed_kmh, 0.0);
    }

    #[test]
    fn movement_is_counted_correctly() {
        let history = vec![
            sample(0, 45.0000, 7.0000),
            sample(30, 45.0010, 7.0010),
        ];

        let stats = analyze_movement(
            &history,
            AnalysisPeriod::Custom {
                start_timestamp: 0,
                end_timestamp: 1000,
            },
            test_config(),
        );

        assert!(stats.total_distance_km > 0.0);
        assert_eq!(stats.movement_duration.as_secs(), 30);
        assert_eq!(stats.pause_duration.as_secs(), 0);
        assert!(stats.average_speed_kmh > 0.0);
    }

    #[test]
    fn pause_of_three_minutes_is_counted() {
        let history = vec![
            sample(0, 45.0000, 7.0000),
            sample(30, 45.0000, 7.0000),
            sample(60, 45.0000, 7.0000),
            sample(90, 45.0000, 7.0000),
            sample(120, 45.0000, 7.0000),
            sample(150, 45.0000, 7.0000),
            sample(180, 45.0000, 7.0000),
        ];

        let stats = analyze_movement(
            &history,
            AnalysisPeriod::Custom {
                start_timestamp: 0,
                end_timestamp: 1000,
            },
            test_config(),
        );

        assert_eq!(stats.total_distance_km, 0.0);
        assert_eq!(stats.movement_duration.as_secs(), 0);
        assert_eq!(stats.pause_duration.as_secs(), 180);
        assert_eq!(stats.average_speed_kmh, 0.0);
    }

    #[test]
    fn pause_shorter_than_three_minutes_is_not_counted() {
        let history = vec![
            sample(0, 45.0000, 7.0000),
            sample(60, 45.0000, 7.0000),
        ];

        let stats = analyze_movement(
            &history,
            AnalysisPeriod::Custom {
                start_timestamp: 0,
                end_timestamp: 1000,
            },
            test_config(),
        );

        assert_eq!(stats.total_distance_km, 0.0);
        assert_eq!(stats.movement_duration.as_secs(), 0);
        assert_eq!(stats.pause_duration.as_secs(), 0);
    }

    #[test]
    fn samples_outside_interval_are_ignored() {
        let history = vec![
            sample(0, 45.0000, 7.0000),
            sample(30, 45.0010, 7.0010),
            sample(5000, 46.0000, 8.0000),
        ];

        let stats = analyze_movement(
            &history,
            AnalysisPeriod::Custom {
                start_timestamp: 0,
                end_timestamp: 100,
            },
            test_config(),
        );

        assert_eq!(stats.path.len(), 2);
        assert!(stats.total_distance_km > 0.0);
        assert_eq!(stats.movement_duration.as_secs(), 30);
    }
}