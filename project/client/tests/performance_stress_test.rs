use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use std::time::Duration;
use shared::messages::{Message, AnalyticsField, AnalyticsPeriodMessage};
use shared::update_position::UpdatePosition;
use shared::coordinates::Coordinates;
use chrono::Utc;
use rand::Rng;

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

#[tokio::test]
async fn run_performance_stress_test() {
    println!("Inizio stress test con comportamento randomico e bassa probabilità di sosta...");
    let mut handles = vec![];

    for i in 0..1000 {
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(i * 300)).await;

            let stream_res = TcpStream::connect("127.0.0.1:8080").await;
            let mut stream = match stream_res {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Bot {} fallito connessione: {}", i, e);
                    return;
                }
            };
            
            let username = format!("bot_user_{}", i);
            let password = "password123".to_string();
            let (read_half, mut write_half) = stream.into_split();
            let mut reader = BufReader::new(read_half).lines();

            let reg_msg = Message::Register { 
                username: username.clone(), 
                password: password.clone() 
            };
            let json_reg = serde_json::to_string(&reg_msg).unwrap();
            if write_half.write_all(format!("{json_reg}\n").as_bytes()).await.is_err() {
                return;
            }
            let _ = reader.next_line().await;

            let login_msg = Message::Login { 
                username: username.clone(), 
                password, 
            };
            let json_login = serde_json::to_string(&login_msg).unwrap();
            if write_half.write_all(format!("{json_login}\n").as_bytes()).await.is_err() {
                return;
            }
            let _ = reader.next_line().await;

            let u_clone = username.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    println!("\n[RAW RECEIVED BY {}] ---> {}", u_clone, line);
                }
            });

            let (mut current_lat, mut current_lon) = {
                let mut rng_init = rand::thread_rng();
                (
                    rng_init.gen_range(44.9..=45.2),
                    rng_init.gen_range(7.5..=7.9)
                )
            };

            let mut step_counter = 0;

            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                step_counter += 1;

                // Evento randomico a ogni invio coordinata: probabilità bassa (15%) di fermarsi
                let (distance_meters, bearing_degrees) = {
                    let mut rng = rand::thread_rng();
                    let is_stopped = rng.gen_bool(0.15);
                    let dist = if is_stopped { 0.0 } else { rng.gen_range(10.0..=150.0) };
                    let bearing = rng.gen_range(0.0..360.0);
                    (dist, bearing)
                };

                let (new_lat, new_lon) = move_point_by_meters(
                    current_lat,
                    current_lon,
                    distance_meters,
                    bearing_degrees,
                );
                current_lat = new_lat;
                current_lon = new_lon;

                let time = Utc::now();
                let coordinates = Coordinates::new(
                    current_lat.to_string(), 
                    current_lon.to_string(), 
                    time.timestamp()
                );
                
                let update = UpdatePosition {
                    username: username.clone(),
                    coordinates,
                    time,
                };

                let json_update = serde_json::to_string(&update).unwrap();
                if write_half.write_all(format!("{json_update}\n").as_bytes()).await.is_err() {
                    break;
                }

                if step_counter % 3 == 0 {
                    let broadcast_msg = Message::SendBroadcastMessage { 
                        text: format!("Broadcast da {}", username) 
                    };
                    let json_bc = serde_json::to_string(&broadcast_msg).unwrap();
                    let _ = write_half.write_all(format!("{json_bc}\n").as_bytes()).await;
                }

                if step_counter % 5 == 0 {
                    let analytics_msg = Message::AnalyticsRequest {
                        field: AnalyticsField::All,
                        period: AnalyticsPeriodMessage::Custom {
                            start_timestamp: 0,
                            end_timestamp: i64::MAX,
                        },
                    };
                    let json_an = serde_json::to_string(&analytics_msg).unwrap();
                    let _ = write_half.write_all(format!("{json_an}\n").as_bytes()).await;
                }
            }
        });
        handles.push(handle);
    }

    println!("Stress test avviato con mobilità stocastica (soste a bassa probabilità)...");
    tokio::time::sleep(Duration::from_secs(60)).await;
}