# GeoRust - Manuale Utente

Benvenuti nel manuale utente di **GeoRust**. Nelle prossime righe vedremo insieme come avviare il sistema centrale e come utilizzare l'applicazione Client per simulare il percorso dei veicoli e scambiare messaggi con il resto della flotta.

---

## 1. Prerequisiti e Avvio del Sistema

Il nostro sistema si basa su una classica architettura Client/Server. Questo significa che, prima di poter lanciare il Client a bordo del veicolo, dobbiamo per forza assicurarci che il Server sia già attivo e pronto a ricevere le connessioni.

### 1.1 Avvio del Server Centrale
1. Apri una finestra del terminale.
2. Spostati nella cartella principale del progetto `geo_rust`.
3. Fai partire il server eseguendo il comando:
   `cargo run -p server`
4. Lascia questa finestra aperta in background. Da questo momento, il server inizierà a registrare gli accessi e a processare le posizioni della flotta in tempo reale.

### 1.2 Avvio dell'Applicazione Client
1. Apri una **nuova** finestra del terminale (ricordati di lasciare attiva anche quella del server).
2. Assicurati di trovarti sempre nella cartella principale del progetto.
3. Lancia il comando:
   `cargo run -p client`

> **Nota:** Se provi ad avviare il Client senza aver prima acceso il Server, il sistema ti darà un errore di connessione e si chiuderà da solo.

---

## 2. Autenticazione (Registrazione e Accesso)

Appena avviato il Client, ti comparirà una schermata iniziale che ti offre tre possibilità: 
**1) Register**, 
**2) Login**
**3) Exit**.

* **Registrazione (Opzione 1):** Scegli questa voce se è la prima volta che accedi. Ti chiederemo di inventare un `Username` e una `Password`. 
  
  L'`Username` deve essere lungo dai 3 ai 20 caratteri e può contenere solo lettere senza accenti, numeri, trattini normali (-) o bassi (_). Non sono ammessi spazi o simboli speciali.
  
  La `Password` deve avere almeno 6 caratteri. Quando la digiti non vedrai nulla a schermo: è un comportamento normalissimo e serve per proteggere le tue credenziali da occhi indiscreti.

* **Accesso (Opzione 2):** Dopo esserti registrato, seleziona il Login e inserisci i dati che hai appena creato. Se è tutto corretto, vedrai il messaggio `Login successful! Welcome, [TuoUsername]`.

* **Exit (Opzione 3):** Usa questa opzione semplicemente per chiudere il programma.

---

## 3. Emulazione del Viaggio (Dietro le quinte)

Tutta la simulazione del viaggio avviene in automatico leggendo le coordinate geografiche da un file. Se questo file non esiste ancora, non preoccuparti: ci penserà il Client a crearlo per te.

* **Generazione del percorso:** Subito dopo il tuo primo login, il Client crea una tua cartella personale nel percorso `client/users/[TuoUsername]` e ci inserisce un file chiamato `route.txt` con dentro il tragitto che farà il tuo veicolo.
* **Realismo del traffico:** Per rendere le cose più verosimili, ogni trenta secondi il client genererà una nuova coordinata e la manderà al server. C'è anche un 15% di probabilità che il sistema decida di fare una sosta; altrimenti, si muoverà in direzioni casuali facendo scatti fino a 200 metri alla volta.
* **Invio dati:** Tu non devi fare nulla di particolare. Il programma lavora in background creando la coordinata, inviandola e salvandola nel file rout.txt.

---

## 4. Menu Principale e Statistiche

Mentre continui ad inviare coordinate in background, puoi usare il menu interattivo per controllare le statistiche del tuo viaggio. I calcoli vengono fatti direttamente dal Server:

* **1) Route info:** Ti mostra un riepilogo dei dettagli del tragitto che hai fatto fino a quel momento.
* **2) Average speed:** Calcola e ti dice a che velocità media stai andando.
* **3) Movement duration:** Ti mostra il tempo effettivo in cui sei stato in movimento.
* **4) Pause duration:** Indica quanto tempo totale sei rimasto fermo o in pausa.

Tutti questi dati ti verranno mostrati a schermo nel riquadro `=== Risultato analytics ===` non appena il server li avrà elaborati.

---

## 5. Sistema di Messaggistica (Chat)

Abbiamo inserito anche una funzione per tenerti in contatto in tempo reale con il resto della flotta. Se selezioni la voce **5) Send message**, potrai scegliere tra due modi per comunicare:

* **1) Direct (Messaggio Diretto):** È la classica chat privata. Ti basta scrivere lo username esatto della persona a cui vuoi scrivere e poi digitare il testo del messaggio.
* **2) Broadcast (Messaggio Globale):** Funziona come un megafono per inviare un avviso pubblico a tutti i veicoli collegati in quel momento. Ti basterà scrivere il testo e arriverà a tutta la rete.

**Ricezione Messaggi:**
Quando qualcuno ti scrive, il messaggio comparirà da solo sullo schermo in tempo reale. Capirai subito di che tipo di messaggio si tratta perché vedrai la scritta `[Private message from...]` se è una comunicazione privata, oppure `[Broadcast from...]` se è un avviso generale.

---

## 6. Uscita e Disconnessione

Quando hai finito, ricordati di chiudere la sessione in modo pulito scegliendo l'opzione **6) Logout** dal menu principale. 
Questa operazione è importante perché:
1. Interrompe l'invio delle tue coordinate al server.
2. Ti riporta alla schermata iniziale, così se qualcun altro deve usare il tuo stesso terminale, potrà fare tranquillamente il login con il suo account senza dover riavviare l'intera applicazione.