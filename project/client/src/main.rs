use std::net::TcpStream;
use std::io::BufReader;

mod auth_flow;

fn main() {
    println!("Starting Client Application...");

    let mut stream = match TcpStream::connect("127.0.0.1:8080") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
            return;
        }
    };
    println!("Connect to server!");

    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // Autenticazione
    
    let username = auth_flow::run_auth_flow(&mut stream, &mut reader);

    println!("Authenticated successfully as: {}", username);

    //resto del client (posizioni, messaggi)
}