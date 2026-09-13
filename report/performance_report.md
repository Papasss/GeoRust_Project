# Report sulle Performance del Server (Ambiente macOS)

Il sistema è stato testato su due sistemi operativi differenti con due hardware differenti:

## Primo hardware

* **Sistema Operativo:** macOS
* **Hardware:** MacBook Pro (Chip Apple Silicon M1), 16 GB di RAM
* **Dimensione eseguibile Server:** 1,8 MB
* **Dimensione eseguibile Client:** 1,6 MB

## Secondo hardware

* **Sistema Operativo:** linux mint
* **Hardware:** iMac 27'' late 20212 (i5-3470), 24 GB di RAM
* **Dimensione eseguibile Server:** 1,8 MB
* **Dimensione eseguibile Client:** 1,6 MB

I test effettuati sono stati di 3 tipi:

* 10 utenti
* 100 utenti
* 1000 utenti

tutti i bot inviavano coordinate periodiche, richieste di analitiche e messaggi broadcast.

---

## Analisi di Esecuzione in idle

Il primo test verifica l'assenza di "busy waiting" (attesa attiva) quando il server è in ascolto senza alcun client connesso.

**Statistiche Medie e Consumi:**
* **Utilizzo CPU medio (%):** ~0.002%
* **Dimensione della Memoria Reale:** 6.8 MB

**Osservazioni:**
I dati confermano che il server in stato di inattività consuma risorse del tutto trascurabili. I task asincroni si sospendono correttamente in attesa di eventi di rete, evitando di sprecare cicli di clock.

---

## Test

### macOS - 10 utenti

**Statistiche Medie e Consumi:**
* **Utilizzo CPU medio (%):** ~0.38% (influenzato dal picco iniziale del 2.08%)
* **Dimensione della Memoria Reale:** 7.9 MB

**Osservazioni:**
Durante l'esecuzione c'è stato un picco di CPU (2.08%) corrispondente alla fase iniziale in cui i 10 client effettuano la registrazione e il server popola in memoria la `HashMap` delle connessioni attive. A login completato, l'elaborazione del traffico a regime (inclusi i calcoli delle analytics e lo smistamento asincrono dei messaggi) richiede uno sforzo minimo alla CPU, assestandosi stabilmente su un fisiologico ~0.05%.

### macOS - 100 utenti

**Statistiche Medie e Consumi:**
* **Utilizzo CPU medio (%):** ~6.80% (con picco al 39.12%)
* **Dimensione della Memoria Reale:** 12 MB

**Osservazioni:**
L'impatto del login simultaneo di 100 client richiede una potenza di elaborazione decisamente maggiore, raggiungendo un utilizzo del 39.11%. Subito dopo questa fase critica, il sistema smaltisce abilmente il tracciamento, le computazioni geografiche e l'invio dei messaggi di broadcast a tutti i 100 utenti, mantenendo la CPU stabilmente attorno allo 0.4%. L'allocazione di memoria cresce in modo proporzionato, raggiungendo i 12 MB per poter mantenere la cronologia e le strutture dati in RAM.

### macOS - 1000 utenti

**Statistiche Medie e Consumi:**
* **Utilizzo CPU medio (%):** ~56.80% (con picco al 222.51%)
* **Dimensione della Memoria Reale:** 118 MB

**Osservazioni:**
A causa dell'elevato livello di concorrenza generato dall'apertura contemporanea di 1000 socket, il login di massa richiede più tempo e il picco si spalma sulle rilevazioni dei minuti successivi. Il processore sfrutta appieno i propri core in parallelo per gestire l'instradamento iniziale, registrando un picco del 222.5% (su 800% di disponibilità massima corrispondente a 8 core).
Una volta conclusa l'onerosa fase iniziale di login e l'allocazione delle risorse, la gestione richiede una media stabile di circa il 7% della CPU. La RAM occupata si stabilizza quindi sui 118 MB per l'intero carico di 1.000 utenti concorrenti (pari a un consumo contenuto di circa 118 KB per sessione attiva). L'assenza di scostamenti o di crescita lineare nel tempo certifica una stabilità dell'applicazione a lungo termine e l'assenza totale di memory leak.

---

### linux-mint - 10 utenti

### linux-mint - 100 utenti

### linux-mint - 1000 utenti