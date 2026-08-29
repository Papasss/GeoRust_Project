use rand::Rng;
use std::path::Path;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;



// Genera un file di testo contenente coordinate e timestamp casuali.
// Scrive il percorso sul file simulando il movimento o la sosta di un utente 
// con intervalli fissi di 30 secondi.

pub async fn create_random_path(file_path: &str) -> std::io::Result<()> {

    let mut file = File::create(file_path).await?;
    let mut rng = rand::thread_rng();
    
    let mut lat_attuale = 45.0618513;
    let mut lon_attuale = 7.6606506;
    let mut tempo_secondi = 0;
    
    for _ in 0..10 {
        let sta_fermo = rng.gen_bool(0.2); 
        
        if !sta_fermo {
            lat_attuale += rng.gen_range(-0.005..0.005);
            lon_attuale += rng.gen_range(-0.005..0.005);
        }
        
        let minuti = tempo_secondi / 60;
        let secondi = tempo_secondi % 60;
        let timestamp = format!("2026-07-08T07:{:02}:{:02}Z", minuti, secondi);
        
        let riga = format!("{},{},{}\n", lat_attuale, lon_attuale, timestamp);
        file.write_all(riga.as_bytes()).await?;
        
        tempo_secondi += 30;
    }

    Ok(())
}



// Verifica l'esistenza della cartella utente e genera il file del percorso se mancante.
// Crea le directory necessarie in modo asincrono e invoca la generazione
// del file con le coordinate fittizie se l'utente accede per la prima volta.

pub async fn ensure_user_path_exists(username: &str, user_dir: &str, file_path: &str) -> std::io::Result<()> {
    
    let path = Path::new(user_dir);

    if !path.exists() {
        println!("\nCreazione della cartella e del percorso di {} ...", username);
        fs::create_dir_all(path).await?;
        create_random_path(file_path).await?;
    } else {
        println!("\nUtente '{}' esistente. Lettura del file esistente...", username);
    }

    Ok(())
}