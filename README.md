# G31

# s362132, s361816, s361837, s364217

# GeoRust - Sistema di Geolocalizzazione Client/Server 🌍🦀

Un sistema di geolocalizzazione e tracciamento flotte sviluppato in Rust. 
Progetto realizzato per il corso di **Programmazione in RUST** presso il **Politecnico di Torino**.

L'applicazione utilizza un'architettura **Client/Server** per gestire il monitoraggio continuo di una flotta di veicoli, l'analisi dei percorsi, le statistiche di movimento e lo scambio di messaggi testuali.

---

## 📋 Indice
1. [Descrizione del Progetto](#-descrizione-del-progetto)
2. [Architettura del Codice](#-architettura-del-codice)
3. [Funzionalità Principali](#-funzionalità-principali)
4. [La Macchina a Stati](#-la-macchina-a-stati)
5. [Requisiti e Installazione](#-requisiti-e-installazione)
6. [Come Eseguire il Progetto](#-come-eseguire-il-progetto)

---

## 🚀 Descrizione del Progetto

L'obiettivo del progetto è tracciare lo stato e la posizione degli utenti (veicoli) registrati a sistema[cite: 1]. 
I veicoli (Client) simulano il proprio movimento inviando aggiornamenti periodici delle loro coordinate GPS[cite: 1]. Il Server centrale riceve i dati, calcola le statistiche di viaggio (tragitto, velocità media, pause) su base giornaliera, settimanale e mensile, e monitora l'utilizzo delle proprie risorse di CPU[cite: 1]. 

Il sistema è multipiattaforma ed è ottimizzato per garantire elevate prestazioni riducendo al minimo la dimensione degli eseguibili[cite: 1].

---

## 🏗 Architettura del Codice

Il progetto sfrutta la funzionalità **Workspace** di Cargo per separare logicamente i componenti. La struttura è divisa in tre *Crate* principali:

*   **`shared/` (Libreria)**: Contiene il "linguaggio comune". Qui sono definite le strutture dati serializzabili (tramite `serde`) usate da entrambe le parti, come `Coordinates`, `UpdatePosition` e gli Enum per gli stati.
*   **`client/` (Eseguibile)**: Il software a bordo del veicolo. Legge un percorso simulato da un file CSV locale e trasmette la posizione esatta al server a intervalli regolari (ogni 30 secondi)[cite: 1]. Permette inoltre di inviare messaggi di testo al server[cite: 1].
*   **`server/` (Eseguibile)**: Il cuore dell'elaborazione. Accetta connessioni in rete tramite socket asincroni (`Tokio`), mantiene in memoria lo storico dei veicoli, gestisce la macchina a stati per le transizioni di movimento e fornisce strumenti di interrogazione per le statistiche.

---

## ✨ Funzionalità Principali

### Lato Client (Emulatore Veicolo)
*   **Registrazione/Login:** Accesso tramite account e password.
*   **Emulazione Movimento:** Lettura di un file di coordinate e timestamp per simulare percorsi realistici (es. Torino -> Asti).
*   **Trasmissione Dati:** Invio asincrono e continuo della posizione ogni 30 secondi.
*   **Comunicazione:** Invio di messaggi testuali verso il server centrale.

### Lato Server (Backend)
*   **Tracciamento Continuo:** Ricezione e immagazzinamento in RAM (`HashMap`) delle coordinate in tempo reale.
*   **Analisi Movimento:** Calcolo di tragitto, velocità media e durata di movimenti/pause.
*   **Comunicazione Bidirezionale:** Invio di messaggi testuali singoli o in broadcast (a tutti gli utenti).
*   **Log Prestazionali:** Generazione automatica di un file di log ogni 2 minuti con i dettagli sul tempo di CPU consumato.

---

## 🚦 La Macchina a Stati (User Tracker)

Il server gestisce lo stato di ogni utente applicando una rigorosa logica temporale basata sulle coordinate ricevute. Gli stati possibili sono tre: **Sconnesso**, **Fermo** e **In Movimento**.

Le regole di transizione implementate sono le seguenti:
1.  **Verso "In Movimento":** Avviene istantaneamente al rilevamento del *primo cambiamento* di coordinata spaziale rispetto all'ultima posizione nota.
2.  **Verso "Fermo":** Avviene esclusivamente quando la coordinata dell'utente *non cambia per almeno 3 minuti consecutivi* (equivalenti a 6 invii da parte del client).

---

## ⚙️ Requisiti e Installazione

Assicurati di avere l'ambiente di sviluppo Rust installato sulla tua macchina. Il progetto è stato testato su sistemi operativi Linux, Windows e MacOS.

```bash
# Clona il repository
git clone [https://github.com/tuo-utente/geo_rust.git](https://github.com/tuo-utente/geo_rust.git)
cd geo_rust

# Compila l'intero Workspace (Client, Server e Shared)
cargo build --release