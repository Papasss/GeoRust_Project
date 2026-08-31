use chrono::{Datelike, Local, TimeZone};
use shared::coordinates::Coordinates;
use std::time::Duration;



#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisPeriod {
    CurrentDay,
    CurrentWeek,
    CurrentMonth,
    Custom {
        start_timestamp: i64,
        end_timestamp: i64,
    },
}



// Configurazione dell'analisi.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalyticsConfig {

    // Tolleranza veicolo fermo.
    
    pub movement_threshold_meters: f64,

    // Durata minima perché una sosta venga contata come pausa.
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



#[derive(Debug, Clone, PartialEq)]
pub struct MovementStatistics {

    // Tragitto percorso: sequenza delle posizioni considerate.
    pub path: Vec<Coordinates>,

    // Distanza totale percorsa in chilometri.
    pub total_distance_km: f64,

    // Durata complessiva del movimento.
    pub movement_duration: Duration,

    // Durata complessiva delle pause.
    pub pause_duration: Duration,

    // Velocità media in km/h.
    pub average_speed_kmh: f64,
}



// Prende la cronologia delle posizioni di un utente, seleziona solo quelle 
// nell'intervallo richiestoe calcola tragitto, distanza, movimento, pause e 
// velocità media.

pub fn analyze_movement( history: &[Coordinates], period: AnalysisPeriod, config: AnalyticsConfig) -> MovementStatistics {
    
    let (start_timestamp, end_timestamp) = resolve_period(period);

    let mut coordinate: Vec<Coordinates> = history
        .iter()
        .filter(|sample| sample.get_timestamp() >= start_timestamp && sample.get_timestamp() <= end_timestamp)
        .cloned()
        .collect();

    coordinate.sort_by_key(|sample| sample.get_timestamp());

    if coordinate.len() < 2 {
        return MovementStatistics {
            path: coordinate,
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

    for pair in coordinate.windows(2) {

        let previous = &pair[0];
        let current = &pair[1];
        let delta_seconds = current.get_timestamp().saturating_sub(previous.get_timestamp());
        let delta_time = Duration::from_secs(delta_seconds.abs_diff(0));

        let distance_km = haversine_distance_km(
            previous.get_latitude(),
            previous.get_longitude(),
            current.get_latitude(),
            current.get_longitude(),
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

        path: coordinate,
        total_distance_km,
        movement_duration,
        pause_duration,
        average_speed_kmh,

    }
}



// Converte CurrentDay / CurrentWeek / CurrentMonth

fn resolve_period(period: AnalysisPeriod) -> (i64, i64) {

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

            (start.timestamp(), now.timestamp())

        }

        AnalysisPeriod::CurrentWeek => {

            let now = Local::now();
            let days_from_monday = now.weekday().num_days_from_monday() as i64;
            let monday = now.date_naive() - chrono::Duration::days(days_from_monday);

            let start = Local
                .with_ymd_and_hms(monday.year(), monday.month(), monday.day(), 0, 0, 0)
                .single()
                .expect("Data locale non valida");

            (start.timestamp(), now.timestamp())

        }

        AnalysisPeriod::CurrentMonth => {

            let now = Local::now();

            let start = Local
                .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
                .single()
                .expect("Data locale non valida");

            (start.timestamp(), now.timestamp())
            
        }
    }
}



// Calcola la distanza in chilometri tra due coordinate geografiche.
// Usa la formula di Haversine.

pub fn haversine_distance_km(lat1: f64,lon1: f64,lat2: f64,lon2: f64) -> f64 {
    
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