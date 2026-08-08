use shared::read_file;
use shared::parse_values;
use std::io::{self, BufRead, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {

    let file = read_file("input.txt")?;
    let reader = BufReader::new(file);

    for line_result in reader.lines() {
        let line = line_result?;
        let values = parse_values(&line);

        if values.is_empty() {
            continue;
        }

    }

    Ok(())
}
