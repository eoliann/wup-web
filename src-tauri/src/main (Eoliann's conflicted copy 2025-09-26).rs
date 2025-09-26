#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;
use tauri::{
    menu::{MenuBuilder, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent, Wry,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_opener::OpenerExt;

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    body: Option<String>,
    html_url: Option<String>,
}

fn fetch_latest_release() -> Result<Release, ()> {
    let client = Client::new();
    let resp = client
        .get("https://api.github.com/repos/eoliann/wup-web/releases/latest")
        .header("User-Agent", "wup-web")
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|_| ())?;

    resp.json::<Release>().map_err(|_| ())
}

fn show_update_dialog(app: &AppHandle<Wry>, current: &str, latest: &str, notes: &str, url: &str) {
    let message = format!(
        "O versiune nouă este disponibilă!\n\nVersiunea curentă: {}\nVersiunea nouă: {}\n\nSchimbări:\n{}",
        current, latest, notes
    );

    app.dialog()
        .message(message)
        .title("Actualizare disponibilă")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom("Update", "Later"))
        .show(move |confirm| {
            if confirm {
                let _ = app.opener().open_url(url, None::<String>);
            }
        });
}

fn check_for_updates(app: AppHandle<Wry>, manual: bool) {
    let current_version_str = env!("CARGO_PKG_VERSION").to_string();
    let current_version =
        Version::parse(&current_version_str).unwrap_or_else(|_| Version::new(0, 0, 0));

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(500));
        let result = fetch_latest_release();

        let _ = app.run_on_main_thread(move || {
            match result {
                Ok(release) => {
                    let latest_str = release.tag_name.trim_start_matches('v').to_string();
                    if let Ok(latest_ver) = Version::parse(&latest_str) {
                        if latest_ver > current_version {
                            let notes = release
                                .body
                                .as_deref()
                                .unwrap_or("Nu există note ale versiunii.");
                            let notes_short = notes.chars().take(1200).collect::<String>();
                            let url = release
                                .html_url
                                .as_deref()
                                .unwrap_or("https://github.com/eoliann/wup-web/releases/latest");
                            show_update_dialog(
                                &app,
                                &current_version_str,
                                &latest_str,
                                &notes_short,
                                url,
                            );
                        } else if manual {
                            app.dialog()
                                .message(format!(
                                    "Aplicația este la zi.\n\nVersiunea curentă: {}",
                                    current_version_str
                                ))
                                .title("Nicio actualizare")
                                .kind(MessageDialogKind::Info)
                                .buttons(MessageDialogButtons::Ok)
                                .show(|_| {});
                        }
                    }
                }
                Err(_) => {
                    if manual {
                        app.dialog()
                            .message("Verificarea actualizărilor a eșuat. Verificați conexiunea.")
                            .title("Eroare")
                            .kind(MessageDialogKind::Error)
                            .buttons(MessageDialogButtons::Ok)
                            .show(|_| {});
                    }
                }
            }
        });
    });
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External("https://web.whatsapp.com/".parse().unwrap()),
            )
            .title("WUP Web")
            .visible(true)
            .build()?;

            let version_item = MenuItem::with_id(
                app,
                "version",
                &format!("Version {}", env!("CARGO_PKG_VERSION")),
                false,
                None::<&str>,
            )?;
            let check_update =
                MenuItem::with_id(app, "check_update", "Check for Updates", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
            let about =
                MenuItem::with_id(app, "about", "About (GitHub)", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = MenuBuilder::new(app)
                .item(&version_item)
                .separator()
                .item(&check_update)
                .separator()
                .item(&show)
                .item(&hide)
                .item(&about)
                .separator()
                .item(&quit)
                .build()?;

            TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .icon_path(app.path().resource_dir().unwrap().join("icons/icon.ico"))
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.hide();
                        }
                    }
                    "about" => {
                        let _ = app.opener().open_url(
                            "https://github.com/eoliann/wup-web",
                            None::<String>,
                        );
                    }
                    "check_update" => check_for_updates(app.clone(), true),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, .. } = event {
                        if button == MouseButton::Left {
                            if let Some(win) =
                                tray.app_handle().get_webview_window("main")
                            {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            let handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(1200));
                check_for_updates(handle, false);
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
