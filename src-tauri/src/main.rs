#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{menu::{Menu, MenuItem}, tray::TrayIconBuilder, Manager, WebviewUrl};

#[tauri::command]
fn show_notification(title: String, body: String) {
    // Momentan doar printăm pentru debugging
    println!("Notification: {} - {}", title, body);
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![show_notification])
        .setup(|app| {
            // Creează fereastra principală WhatsApp la pornire
            let window = tauri::WebviewWindowBuilder::new(app, "whatsapp", WebviewUrl::External("https://web.whatsapp.com".parse().unwrap()))
                .title("WhatsApp Web")
                .inner_size(1200.0, 800.0)
                .build()?;
            
            // Injectează script pentru detectare notificări
            let script = r#"
                (function() {
                    let lastTitle = document.title;
                    setInterval(() => {
                        if (document.title !== lastTitle) {
                            lastTitle = document.title;
                            if (lastTitle.includes("(") && lastTitle.includes(")")) {
                                // Probabil notificare nouă
                                window.__TAURI__.invoke('show_notification', {
                                    title: 'WhatsApp Web',
                                    body: lastTitle
                                });
                            }
                        }
                    }, 1000);
                })();
            "#;
            
            window.eval(script).unwrap();
            
            // Creează itemii pentru tray menu
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show WhatsApp", true, None::<&str>)?;
            let open_business = MenuItem::with_id(app, "open_business", "Open WhatsApp Business", true, None::<&str>)?;
            
            // Creează meniul
            let menu = Menu::with_items(app, &[&show, &open_business, &hide, &quit])?;
            
            // Creează tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("whatsapp") {
                            window.hide().unwrap();
                        }
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("whatsapp") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    "open_business" => {
                        // Creează fereastra Business
                        if let Some(window) = app.get_webview_window("business") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        } else {
                            tauri::WebviewWindowBuilder::new(app, "business", WebviewUrl::External("https://web.whatsapp.com".parse().unwrap()))
                                .title("WhatsApp Business")
                                .inner_size(1200.0, 800.0)
                                .build()
                                .unwrap();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, _event| {});
}