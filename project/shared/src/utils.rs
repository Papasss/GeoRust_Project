use serde::Serialize;
use tokio::io::{AsyncWrite, AsyncWriteExt};



// Serializza un pacchetto dati in formato JSON e lo trasmette sul socket di rete.
// Converte la struttura generica in byte e aggiunge un carattere di a capo finale 
// per segnalare la fine del messaggio, garantendo l'invio asincrono senza blocchi.
pub async fn send_packet<T, W>(writer: &mut W, packet: &T) -> std::io::Result<()>
where
    T: Serialize,
    W: AsyncWrite + Unpin,
{
    let json_data = serde_json::to_string(packet)
        .expect("Errore critico nella serializzazione JSON");
        
    writer.write_all(format!("{}\n", json_data).as_bytes()).await?;
    writer.flush().await
}