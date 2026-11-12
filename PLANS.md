# Roadmap Wup Web

## 1. Validare tehnică — în lucru

Verificarea cu conturi reale a fost mutată la final, la cererea utilizatorului.
Versiunea 2.0.0 include implementarea de test pentru patru conturi, notificările
implicite WebView2, contorul din titlul paginii și updaterul GitHub după modelul
Eoliann Windows Tools. Testul WebView2 izolat a verificat butoanele și comutarea
1 → 2 → 1. Începând cu v2.0.0, publicăm doar installerul Inno Setup x64.
Am verificat descărcarea installerului public și compararea SHA-256 cu digestul
GitHub. Scenariile cu conturi reale rămân pentru validare.

Progres. Configurația Codex a fost creată și verificată ca TOML. Verificarea `cargo check --locked` a trecut cu un cache Cargo temporar, separat de cel global. Proba locală Wup Web Probe a trecut buildul și testul de extragere a numărului din titlu. La pornire, ambele profiluri de test au fost create, au încărcat WhatsApp Web și au înregistrat observatorii WebView2.

Pentru 2.1.0, utilizatorul a confirmat în Windows, cu sesiunea existentă,
atașarea unui fișier prin drag and drop, redenumirea contului, sincronizarea
pozei de profil și păstrarea numelui și pictogramei după redeschidere. Mai rămâne
proba de upgrade cu installerul. Verificările cu două sesiuni așteaptă un al
doilea cont disponibil.

Următoarea acțiune. Verifică scenariile de mai jos când există două conturi de
test disponibile. Rezultatele privind contorul exact, livrarea alertelor și
persistența mai multor sesiuni autentificate sunt încă deschise.

- Verifică buildul versiunii actuale cu lockfile-ul din repository.
- Pregătește două profiluri de test separate de profilul curent. Verifică persistența lor și funcționarea contului neselectat când aplicația este în tray.
- Trimite trei mesaje în aceeași conversație, apoi citește-le pe telefon. Verifică dacă sursa candidat raportează 3, apoi 0. Repetă cu o conversație silențioasă.
- Verifică semnalele pentru mesaj, reacție la un mesaj vechi, apel audio și apel video, cu contul vizibil și ascuns. Pentru testele reale sunt necesare două conturi conectate și un expeditor.
- Închide etapa cu o sursă verificată pentru fiecare cerință sau cu un blocaj concret discutat înainte de a continua.

## 2. Conturi și comutare

După validarea izolării, implementează profilurile persistente și selectorul. Verifică două conturi după comutare, repornire și rulare în tray. Testează actualizarea cu o sesiune existentă.

## 3. Notificări Windows

Integrează semnalele confirmate. Verifică mesajele, reacțiile și apelurile audio/video pe contul selectat și pe cel neselectat, inclusiv în tray. Verifică lipsa duplicatelor, selectarea contului corect la clic și comportamentul în prim-plan față de aplicația oficială.

## 4. Contor pe ambele pictograme

Folosește sursa validată pentru contul selectat. Cu trei necitite în A și două în B, ambele pictograme arată trei pe A și două după comutare. Verifică citirea de pe telefon, conversațiile silențioase, revenirea la zero și restaurarea ferestrei.

## 5. Patru conturi și distribuție

Verifică patru sesiuni simultane și limita de adăugare. Verifică notificările de la toate conturile, contorul contului selectat, restartul, tray-ul și autostartul după instalare și după upgrade.
