# GeoRust

**Sistema di Geolocalizzazione Client/Server**

*Progetto realizzato per il corso di Programmazione di Sistema (modulo Rust) presso il Politecnico di Torino.*
*Gruppo G31: s362132, s361816, s361837, s364217*

Abbiamo costruito questo sistema per monitorare flotte di veicoli in tempo reale. L'idea di base è che ogni veicolo (il Client) simula un viaggio reale inviando aggiornamenti GPS continui. Dall'altra parte, il Server centrale raccoglie queste posizioni, calcola statistiche come la velocità media e i tempi di sosta, e gestisce lo scambio di messaggi di testo tra i conducenti e la centrale. 

Ci siamo concentrati molto sulle performance, cercando di mantenere l'applicativo leggero e multipiattaforma (Linux, Windows, macOS), tenendo sempre d'occhio l'uso effettivo della CPU.

---

## Indice
1. [Architettura del Codice](#architettura-del-codice)
2. [Funzionalità Principali](#funzionalità-principali)
3. [Logica di Tracking](#logica-di-tracking)
4. [Requisiti e Installazione](#requisiti-e-installazione)

---

## Architettura del Codice

Per mantenere il codice pulito, abbiamo strutturato il progetto usando un **Cargo Workspace** diviso in tre moduli principali:

* **`shared/`**: È la libreria condivisa, il vero e proprio vocabolario comune tra client e server. Qui dentro si trovano i tipi base e le strutture dati serializzabili (grazie a `serde`), come le coordinate geografiche e i pacchetti dei messaggi.
* **`client/`**: Il software di bordo. Legge un percorso simulato da un file locale e invia un "ping" con la posizione esatta al server ogni 30 secondi. Permette inoltre di chattare in tempo reale.
* **`server/`**: Il motore del sistema. Gestisce le connessioni di rete appoggiandosi ai socket asincroni di Tokio, tiene traccia dei veicoli in memoria e smista la messaggistica istantanea in modo concorrente.

---

## Funzionalità Principali

**Lato Client (Veicolo)**
* **Login & Registrazione:** Accesso sicuro tramite credenziali.
* **Simulazione Movimento:** Usa un file locale con coordinate e timestamp per simulare viaggi realistici.
* **Tracking Automatico:** Spara la posizione in background ogni 30 secondi, lasciando sempre libera e reattiva l'interfaccia.
* **Chat:** Scambio di messaggi di testo diretti o in broadcast verso gli altri veicoli.

**Lato Server (Centrale)**
* **Tracciamento:** Mantiene lo storico e le posizioni attuali della flotta in RAM (tramite una `HashMap`) per un accesso rapidissimo.
* **Analytics:** Calcola le distanze, la velocità media e la durata effettiva di pause e movimenti su scala giornaliera, settimanale e mensile.
* **Messaging:** Fa da postino smistando in sicurezza i messaggi asincroni tra i vari client connessi.
* **Monitoraggio CPU:** Un demone in background registra automaticamente su file il tempo di CPU consumato dall'applicativo, con uno scatto ogni 2 minuti.

---

## Logica di Tracking

Il server non si limita a salvare le coordinate, ma deduce cosa sta facendo il veicolo tramite una piccola macchina a stati temporale. Un veicolo può trovarsi in tre situazioni: **Sconnesso**, **Fermo** e **In Movimento**.

Le regole di transizione sono semplici:
* **Diventare "In Movimento":** Scatta al volo, non appena il server riceve una coordinata spaziale diversa dall'ultima registrata.
* **Diventare "Fermo":** Per evitare falsi parcheggi, il server considera un veicolo effettivamente fermo solo se riceve la stessa identica coordinata per almeno 3 minuti.

---

## Requisiti e Installazione

Per compilare ed eseguire il progetto sono necessari:

* [Rust e Cargo](https://www.rust-lang.org/tools/install), con una versione compatibile con Rust edition 2024;
* una connessione di rete locale tra client e server, se vengono eseguiti su macchine diverse.

Il progetto si trova nella directory `project/code/`, che contiene il workspace Cargo. Per compilare tutti i moduli:

```bash
cd project/code
cargo build
```

Per eseguire i test:

```bash
cargo test
```

Per avviare il server o il client:

```bash
cargo run -p server
cargo run -p client
```

#### Il client utilizza i file di dati e i percorsi presenti nella sua directory di progetto; prima dell'avvio assicurarsi che tali file siano disponibili nel percorso previsto dall'applicazione.
---