# Wup Web

- Păstrează Rust, Tauri și WebView2. Separă gestionarea conturilor, integrarea WhatsApp Web, integrarea Windows și controalele locale de comutare.
- Folosește un profil persistent și un ID stabil pentru fiecare cont, maximum patru. Direcționează notificările după contul sursă. Ambele pictograme afișează numărul de mesaje individuale necitite din contul selectat.
- Menține pagina WhatsApp Web separată de comenzile locale privilegiate. Exclude conținutul mesajelor, codurile QR și datele sesiunilor din loguri, documentație și datele de test din repository.
- Urmează convențiile Rust existente și limitează modificările la etapa aprobată. Păstrează rularea în tray și autostartul. Release-urile publice distribuie installerul x64 sub numele Wup_Web_Installer.exe. Cere aprobare pentru schimbări de arhitectură și publicare.
- Pentru schimbări Rust, rulează verificările relevante de formatare, compilare și test. Verifică notificările, contorul și sesiunile și într-o sesiune grafică Windows cu conturi de test. Raportează separat scenariile neexecutate.
- La reluare, citește etapa activă din PLANS.md și deciziile relevante din MEMORY.md. La final, raportează schimbările, fișierele, verificările executate, problemele rămase și următorul pas.
