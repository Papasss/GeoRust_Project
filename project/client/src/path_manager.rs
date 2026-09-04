use rand::Rng;
use std::path::Path;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;



// Generates a text file containing random coordinates and timestamps.
// It simulates a user moving or stopping with fixed 30-second intervals.

fn random_start_point(rng: &mut rand::rngs::ThreadRng) -> (f64, f64) {
    let min_latitude = 44.9;
    let max_latitude = 45.2;
    let min_longitude = 7.5;
    let max_longitude = 7.9;

    let latitude = rng.gen_range(min_latitude..=max_latitude);
    let longitude = rng.gen_range(min_longitude..=max_longitude);

    (latitude, longitude)
}

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

pub async fn create_random_path(file_path: &str) -> std::io::Result<()> {
    let mut file = File::create(file_path).await?;
    let mut rng = rand::thread_rng();

    let mut current_latitude;
    let mut current_longitude;
    let mut elapsed_seconds = 0;
    let total_points = rng.gen_range(10..=50);

    (current_latitude, current_longitude) = random_start_point(&mut rng);

    for _ in 0..total_points {
        let is_stopped = rng.gen_bool(0.15);
        let distance_meters = if is_stopped {
            0.0
        } else {
            rng.gen_range(0.0..=200.0)
        };

        let bearing_degrees = rng.gen_range(0.0..360.0);
        let (new_latitude, new_longitude) =
            move_point_by_meters(current_latitude, current_longitude, distance_meters, bearing_degrees);
        current_latitude = new_latitude;
        current_longitude = new_longitude;

        let minutes = elapsed_seconds / 60;
        let seconds = elapsed_seconds % 60;
        let timestamp = format!("2026-07-08T07:{:02}:{:02}Z", minutes, seconds);

        let row = format!("{},{},{}\n", current_latitude, current_longitude, timestamp);
        file.write_all(row.as_bytes()).await?;

        elapsed_seconds += 30;
    }

    Ok(())
}



// Verifica l'esistenza della cartella utente e genera il file del percorso se mancante.
// Crea le directory necessarie in modo asincrono e invoca la generazione
// del file con le coordinate fittizie se l'utente accede per la prima volta.

pub async fn ensure_user_path_exists(username: &str, user_dir: &str, file_path: &str) -> std::io::Result<()> {
    
    let path = Path::new(user_dir);

    if !path.exists() {
        println!("\nCreating the user folder and path for {} ...", username);
        fs::create_dir_all(path).await?;
        create_random_path(file_path).await?;
    } else {
        println!("\nUser '{}' exists. Reading existing file...", username);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn haversine_meters(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let r = 6_371_000.0;
        let lat1 = lat1.to_radians();
        let lat2 = lat2.to_radians();
        let dlat = (lat2 - lat1) / 2.0;
        let dlon = (lon2 - lon1).to_radians() / 2.0;
        let a = (dlat.sin().powi(2)) + lat1.cos() * lat2.cos() * dlon.sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        r * c
    }

    #[tokio::test]
    async fn create_random_path_generates_dynamic_route_with_coherent_steps() {
        let dir = std::env::temp_dir().join("random_path_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("route.txt");

        create_random_path(file_path.to_str().unwrap()).await.unwrap();

        let contents = fs::read_to_string(&file_path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert!(lines.len() >= 10 && lines.len() <= 50, "unexpected point count: {}", lines.len());

        let mut previous: Option<(f64, f64)> = None;
        for line in lines {
            let mut parts = line.split(',');
            let lat = parts.next().unwrap().parse::<f64>().unwrap();
            let lon = parts.next().unwrap().parse::<f64>().unwrap();

            if let Some((prev_lat, prev_lon)) = previous {
                let distance = haversine_meters(prev_lat, prev_lon, lat, lon);
                assert!(distance <= 200.0, "jump too long: {}m", distance);
            }

            previous = Some((lat, lon));
        }
    }
}