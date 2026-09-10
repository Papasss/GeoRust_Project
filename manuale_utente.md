# 🚗 GeoRust - Manuale Utente

Benvenuto nel Manuale Utente del sistema **GeoRust**. Questo documento ti guiderà passo dopo passo nell'avvio del sistema centrale e nell'utilizzo dell'applicazione Client per simulare il movimento dei veicoli e comunicare con la flotta.

---

## ⚙️ 1. Prerequisiti e Avvio del Sistema

Il sistema si basa su un'architettura Client/Server. Affinché il Client possa funzionare, è strettamente necessario che il Server centrale sia attivo e in ascolto.

### 1.1 Avvio del Server Centrale
1. Apri una finestra del terminale.
2. Naviga nella cartella principale del progetto `geo_rust`.
3. Avvia il server eseguendo il comando:
   `cargo run -p server`
4. Lascia questa finestra del terminale aperta in background. Il server inizierà a registrare gli accessi e a processare le posizioni.

### 1.2 Avvio dell'Applicazione Client
1. Apri una **nuova** finestra del terminale (mantenendo attiva quella del server).
2. Assicurati di essere sempre nella cartella principale del progetto.
3. Esegui il comando:
   `cargo run -p client`
> **Nota:** Se avvii il Client senza aver prima avviato il Server, il sistema segnalerà un errore di connessione e si chiuderà in automatico.

---

## 🔐 2. Autenticazione (Registrazione e Accesso)

All'avvio del Client, ti troverai di fronte alla schermata iniziale con tre opzioni: **1) Register**, **2) Login** e **3) Exit**.

* **Registrazione (Opzione 1):**
  Seleziona questa opzione al tuo primo accesso. Ti verrà richiesto di digitare un `Username` e una `Password`. Per la tua sicurezza, i caratteri della password non verranno mostrati a schermo durante la digitazione.
* **Accesso (Opzione 2):**
  Una volta completata la registrazione, seleziona l'opzione di Login e inserisci le tue credenziali appena create. Il sistema confermerà l'accesso con il messaggio `Login successful! Welcome, [TuoUsername]`.
* **Exit (Opzione 3):**
  Seleziona questa opzione per uscire dall'applicazione.

---

## 🗺️ 3. Emulazione del Viaggio (Dietro le quinte)

Il sistema simula il movimento del veicolo in modo del tutto automatico, partendo da un file con le coordinate geografiche dell'utente. Nel caso in cui il file non esista, il Client lo genererà automaticamente.

* **Generazione del percorso:** Dopo il tuo primo accesso, il Client creerà una cartella personale nel percorso `client/users/[TuoUsername]` e genererà un file denominato `route.txt` contenente il tragitto del tuo veicolo. 
* **Realismo del traffico:** Il percorso comprende tra le 10 e le 50 posizioni geografiche. Esiste una probabilità del 15% che il veicolo simuli una sosta; nei restanti casi, si muoverà in direzioni casuali percorrendo fino a 200 metri alla volta.
* **Invio dati:** L'applicazione lavora in background leggendo il file generato e trasmettendo la posizione al server ogni 30 secondi esatti.

---

## 📊 4. Menu Principale e Statistiche

Mentre il veicolo "viaggia" in background, hai a disposizione un menu interattivo per monitorare le tue statistiche di viaggio, elaborate direttamente dal Server:

* **1) Route info:** Mostra il dettaglio del tragitto percorso finora.
* **2) Average speed:** Calcola e restituisce la velocità media attuale del veicolo.
* **3) Movement duration:** Indica il tempo totale (complessivo) che hai trascorso in movimento.
* **4) Pause duration:** Indica il tempo totale (complessivo) trascorso in stato di sosta/fermo.

I risultati richiesti verranno stampati a schermo non appena il server li avrà elaborati, all'interno del riquadro `=== Risultato analytics ===`.

---

## 💬 5. Sistema di Messaggistica (Chat)

Il Client integra una funzione per comunicare in tempo reale con gli altri veicoli della flotta. Scegliendo l'opzione **5) Send message** accederai a due modalità di invio:

* **1) Direct (Messaggio Diretto):** Permette di inviare un messaggio privato. Dovrai specificare lo username esatto del destinatario e successivamente il testo.
* **2) Broadcast (Messaggio Globale):** Permette di inviare un avviso pubblico a tutti i veicoli attualmente connessi alla rete. Ti basterà inserire il testo del messaggio.

**Ricezione Messaggi:**
I messaggi in arrivo compariranno automaticamente e in tempo reale sul tuo schermo. Saranno facilmente identificabili grazie alle diciture `[Private message from...]` per le comunicazioni dirette e `[Broadcast from...]` per gli avvisi globali.

---

## 🚪 6. Uscita e Disconnessione

Per chiudere la tua sessione in modo pulito e sicuro, seleziona l'opzione **6) Logout** dal menu principale.
Questa azione:
1. Interrompe l'invio delle tue coordinate al server.
2. Ti riporta alla schermata di benvenuto, permettendo a un altro utente di effettuare il Login sullo stesso terminale senza dover riavviare l'applicazione.