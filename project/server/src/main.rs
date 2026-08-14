mod serverState;

use std::io;
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepted connection from {}", addr);

        tokio::spawn(async move {
            let mut reader = BufReader::new(socket);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        println!("Client {} disconnected", addr);
                        break;
                    }
                    Ok(_) => {
                        print!("[{}] {}", addr, line);
                    }
                    Err(e) => {
                        eprintln!("Error reading from {}: {}", addr, e);
                        break;
                    }
                }
            }
        });
    }

}
