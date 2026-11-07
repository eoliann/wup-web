# Decizii confirmate

- Wup Web rămâne o aplicație Windows în Rust, Tauri și WebView2. Începând cu 2.0.0, distribuția publică folosește doar installerul x64, cu numele stabil Wup_Web_Installer.exe.
- Aplicația va suporta maximum patru conturi conectate simultan, fiecare cu un profil WebView2 persistent separat și un ID stabil. Un singur cont este afișat la un moment dat.
- Notificările Windows acoperă mesaje noi, reacții și apeluri audio/video de la toate conturile conectate. Comportamentul aplicației oficiale WhatsApp pentru Windows este reperul.
- Ambele pictograme, din bara de activități și din tray, afișează totalul mesajelor individuale necitite din contul selectat.
- ZapZap servește ca referință de arhitectură. Implementarea Wup Web este proprie, sub licența MIT a proiectului.
- Validăm sursa contorului și evenimentele din fundal cu două sesiuni înainte de a extinde la patru. O schimbare a cerințelor cere aprobarea utilizatorului.
