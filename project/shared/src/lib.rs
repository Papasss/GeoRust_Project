use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::process;

pub mod messages;
pub mod coordinates;
pub mod user_state;
pub mod update_position;

pub fn read_file(path: &str) -> io::Result<File> {

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            match e.kind() {
                ErrorKind::PermissionDenied => {
                    eprintln!("Permission denied");
                    process::exit(1);
                }
                ErrorKind::NotFound => {
                    eprintln!("File does not exist");
                    process::exit(1);
                }
                _ => {
                    eprintln!("Generic error: {}", e); 
                    process::exit(1);
                }
            } 
        }
    };

    return Ok(file);
}


pub fn parse_values(line: &str) -> Vec<String> {
    line
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

