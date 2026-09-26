use rand::Rng;
use chrono::{DateTime, Utc};
use shared::{coordinates::Coordinates, update_position::UpdatePosition};
use std::path::Path;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::AsyncWriteExt;



// Genera un punto di partenza casuale per un nuovo tracciato.

fn random_start_point(rng: &mut rand::rngs::ThreadRng) -> (f64, f64) {

    let min_latitude = 44.9;
    let max_latitude = 45.2;
    let min_longitude = 7.5;
    let max_longitude = 7.9;

    let latitude = rng.gen_range(min_latitude..=max_latitude);
    let longitude = rng.gen_range(min_longitude..=max_longitude);

    (latitude, longitude)
}



// Calcola la nuova coordinata geografica simulando lo spostamento di un veicolo.

fn move_point_by_meters(lat: f64, lon: f64, distance_m: f64, bearing_deg: f64) -> (f64, f64) {

    const EARTH_RADIUS_M: f64 = 6_371_000.0;
    let bearing = bearing_deg.to_radians();
    let angular_distance = distance_m / EARTH_RADIUS_M;

    let lat1 = lat.to_radians();
    let lon1 = lon.to_radians();

    let lat2 = (lat1.sin() * angular_distance.cos() +
        lat1.cos() * angular_distance.sin() * bearing.cos())
        .asin();

    let lon2 = lon1
        + (bearing.sin() * angular_distance.sin() * lat1.cos())
            .atan2(angular_distance.cos() - lat1.sin() * lat2.sin());

    (lat2.to_degrees(), lon2.to_degrees())
}



// Genera una nuova posizione basandosi sulla precedente e la accoda al file locale.

pub async fn append_random_position(
    file_path: &str,
    username: &str,
    previous_position: Option<(f64, f64)>,
) -> std::io::Result<((f64, f64), UpdatePosition)> {

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .await?;

    let position = {
        let mut rng = rand::thread_rng();
        let (current_latitude, current_longitude) = previous_position
            .unwrap_or_else(|| random_start_point(&mut rng));
        let is_stopped = rng.gen_bool(0.15);
        let distance_meters = if is_stopped { 0.0 } else { rng.gen_range(0.0..=200.0) };
        let bearing_degrees = rng.gen_range(0.0..360.0);

        move_point_by_meters(
            current_latitude,
            current_longitude,
            distance_meters,
            bearing_degrees,
        )
    };

    let time: DateTime<Utc> = Utc::now();
    let row = format!("{},{},{}\n", position.0, position.1, time.to_rfc3339());
    file.write_all(row.as_bytes()).await?;

    let coordinates = Coordinates::new(position.0.to_string(), position.1.to_string(), time.timestamp());
    
    let update = UpdatePosition {
        username: username.to_string(),
        coordinates,
        time,
    };

    Ok((position, update))
}



// Controlla l'esistenza del path utente e inizializza i file necessari.

pub async fn ensure_user_path_exists(username: &str, user_dir: &str, file_path: &str) -> std::io::Result<()> {
    
    let path = Path::new(user_dir);

    if !path.exists() {
        
        println!("\nCreating the user folder and simulated path for \x1b[36m{}\x1b[0m...", username);
        fs::create_dir_all(path).await?;
        File::create(file_path).await?;
        
    } else {
        
        println!("\nUser '\x1b[36m{}\x1b[0m' exists. Reading existing route file...", username);
        if !Path::new(file_path).exists() {
            File::create(file_path).await?;
        }

    }

    Ok(())
}