![Followers](https://img.shields.io/github/followers/eoliann?style=plastic&color=green)
![Watchers](https://img.shields.io/github/watchers/eoliann/wup-web?style=plastic)
![Stars](https://img.shields.io/github/stars/eoliann/wup-web?style=plastic)
[![Donate](https://img.shields.io/badge/Donate-PayPal-blue?style=plastic)](https://www.paypal.com/donate/?hosted_button_id=PTH2EXUDS423S)
[![Donate](https://img.shields.io/badge/Donate-Revolut-8A2BE2?style=plastic)](https://revolut.me/adriannm9)
[![Donate](https://img.shields.io/badge/Donate-KoFi-green?style=plastic)](https://ko-fi.com/eoliann)


![Release Date](https://img.shields.io/github/release-date/eoliann/wup-web?style=plastic)
![Last Commit](https://img.shields.io/github/last-commit/eoliann/wup-web?style=plastic)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg?style=plastic)](LICENSE.md)
![OS](https://img.shields.io/badge/OS-Windows-blue?style=plastic)
![Lang](https://img.shields.io/badge/Lang-Rust-magenta?style=plastic)

![Total Downloads](https://img.shields.io/github/downloads/eoliann/wup-web/total?style=plastic)
[![Downloads latest](https://img.shields.io/github/downloads/eoliann/wup-web/latest/total?style=plastic)](https://github.com/eoliann/wup-web/releases/latest)
[![Installer downloads](https://img.shields.io/github/downloads/eoliann/wup-web/latest/Wup_Web_Installer.exe?style=plastic&label=installer%20downloads)](https://github.com/eoliann/wup-web/releases/latest/download/Wup_Web_Installer.exe)

# Wup Web

A Windows desktop client for WhatsApp Web, with persistent sessions, a system
tray icon and optional Windows autostart.
<img width="1010" height="903" alt="Captură de ecran 2026-11-07 115645" src="https://github.com/user-attachments/assets/ae5ce351-220f-4cb7-825d-4b8f8d57d874" />


## Download

- [Download the Wup Web installer (Windows x64)](https://github.com/eoliann/wup-web/releases/latest/download/Wup_Web_Installer.exe)
- [Latest release and release notes](https://github.com/eoliann/wup-web/releases/latest)

Wup Web uses the Microsoft Edge WebView2 Runtime.

## What's new in 2.0.1

- Each account button shows its own unread count, including for accounts in
  the background. The badge clears when that account has no unread items.

## What's new in 2.0.0

- **Four accounts.** Link each account in a separate slot. Use the buttons
  above the conversations to switch accounts in one click. The active account
  is highlighted, and the other sessions stay open in the background.
- **Desktop notifications.** Enable desktop notifications in WhatsApp Web for
  each account. Windows alerts depend on the events WhatsApp Web delivers to
  WebView2 and on your notification settings.
- **Unread badge.** The taskbar and tray display the selected account's
  WhatsApp Web unread indicator, in pale yellow on dark green.
- **GitHub updates.** Wup Web checks at startup and from **Check for updates**
  in the window or tray menu. Applying an update requires confirmation and
  verification against the installer's GitHub SHA-256 digest.

## Local build

On Windows, install the Rust MSVC toolchain and the Visual Studio C++ build
tools. From the repository root, run:

```powershell
cargo build --release --locked --manifest-path src-tauri/Cargo.toml
cargo test --locked --manifest-path src-tauri/Cargo.toml
```

Build the installer with Inno Setup using `installer/WupWeb.iss` and pass
`src-tauri/target/release/wup-web.exe` as the input binary.

---

## 📄 License

MIT

---

## 🛠 Bug-uri i erori / Bugs and Errors
- If you discover any bugs or errors, please send them to my channel at **[Discord](https://discord.com/channels/977086560782663680/1416056622069055580)**
- Documentation <a href="https://discord.com/channels/977086560782663680/1416056737047253156" target="_blank"><img src="https://img.shields.io/badge/documentation-available-blue?logo=readthedocs" alt="Documentation"/></a>

---

## 💖 Support
- If you find the project helpful, please consider giving it a ⭐️ to show your support!
- Support this project with a donation
  - Revolut [![Donate](https://img.shields.io/badge/Donate-Revolut-purple)](http://revolut.me/adriannm9)
  - PayPal [![Donate](https://img.shields.io/badge/Donate-PayPal-blue)](https://www.paypal.com/donate/?hosted_button_id=U9XAX3XBTU67G)

