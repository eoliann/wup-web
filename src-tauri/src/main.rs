#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::{Arc, Mutex};

use tauri::{
    generate_context, Manager, Wry,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
    WindowEvent,
};

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

const APP_NAME: &str = env!("CARGO_PKG_NAME"); // valoarea folosită în registry

/// Escape simplu pentru a pune un text într-un literal JS single-quoted.
fn js_escape_single_quote(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('\'', "\\'")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
}

#[cfg(windows)]
fn check_autostart() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
        Ok(run_key) => run_key.get_value::<String, &str>(APP_NAME).is_ok(),
        Err(_) => false,
    }
}

#[cfg(not(windows))]
fn check_autostart() -> bool {
    false
}

#[cfg(windows)]
fn enable_autostart_impl() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_str = exe.to_str().ok_or("Invalid exe path")?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run_key, _disp) = hkcu
        .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
        .map_err(|e| e.to_string())?;
    run_key
        .set_value(APP_NAME, &format!("\"{}\"", exe_str))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(windows))]
fn enable_autostart_impl() -> Result<(), String> {
    Err("Autostart not implemented on this platform".into())
}

#[cfg(windows)]
fn disable_autostart_impl() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_WRITE,
    ) {
        Ok(run_key) => {
            let _ = run_key.delete_value(APP_NAME);
            Ok(())
        }
        Err(_) => Ok(()),
    }
}

#[cfg(not(windows))]
fn disable_autostart_impl() -> Result<(), String> {
    Err("Autostart not implemented on this platform".into())
}

// Tauri commands (frontend can invoke these if needed)
#[tauri::command]
fn is_autostart_enabled() -> Result<bool, String> {
    Ok(check_autostart())
}
#[tauri::command]
fn enable_autostart() -> Result<(), String> {
    enable_autostart_impl()
}
#[tauri::command]
fn disable_autostart() -> Result<(), String> {
    disable_autostart_impl()
}

// Build a Menu<Wry> (note explicit generic)
fn build_menu_for(app: &tauri::AppHandle<Wry>, autostart_on: bool, _product: &str, version_label: &str) -> Menu<Wry> {
    let autostart_label = if autostart_on { "Autostart: On" } else { "Autostart: Off" };

    let version_item: MenuItem<Wry> = MenuItem::with_id(
        app,
        "version",
        &format!("Version: {}", version_label),
        true,
        None::<&str>,
    ).expect("failed to create version item");

    let sep_item: MenuItem<Wry> = MenuItem::with_id(app, "sep", "──────────", false, None::<&str>)
        .expect("failed to create sep");

    let autostart_item: MenuItem<Wry> = MenuItem::with_id(app, "autostart", autostart_label, true, None::<&str>)
        .expect("failed to create autostart");

    let show: MenuItem<Wry> = MenuItem::with_id(app, "show", "Open", true, None::<&str>)
        .expect("failed to create show");
    let quit: MenuItem<Wry> = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)
        .expect("failed to create quit");

    Menu::with_items(app, &[&version_item, &sep_item, &autostart_item, &show, &quit])
        .expect("failed to build menu")
}

fn main() {
    let context = generate_context!();

    // product name din tauri.conf.json (productName)
    let product_name: String = context
        .config()
        .product_name
        .clone()
        .unwrap_or_else(|| context.package_info().name.clone());

    // versiune din Cargo.toml la compilare
    let version_label = format!("{}", env!("CARGO_PKG_VERSION"));

    // initial autostart state (read from registry)
    let initial_autostart = check_autostart();

    // shared state so closures can toggle and read
    let autostart_state = Arc::new(Mutex::new(initial_autostart));

    tauri::Builder::default()
        // expunem comenzile pentru frontend (opțional)
        .invoke_handler(tauri::generate_handler![
            is_autostart_enabled,
            enable_autostart,
            disable_autostart
        ])
        .setup(move |app| {
            // Build initial menu using current autostart state
            let menu = build_menu_for(&app.app_handle(), initial_autostart, &product_name, &version_label);

            // clone needed things for closures
            let product_for_closure = product_name.clone();
            let version_for_closure = version_label.clone();
            let autostart_state_for_menu = autostart_state.clone();

            // Tray icon + handlers
            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.unminimize();
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "version" => {
                            let changelog_text = "Placeholder changelog — înlocuiește cu textul real din CHANGELOG.md";
                            let p = js_escape_single_quote(&product_for_closure);
                            let v = js_escape_single_quote(&version_for_closure);
                            let c = js_escape_single_quote(changelog_text);

                            let js = format!(
                                "window.dispatchEvent(new CustomEvent('show-changelog', {{ detail: {{ product: '{}', version: '{}', changelog: '{}' }} }}));",
                                p, v, c
                            );

                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.eval(&js);
                            }
                        }
                        "autostart" => {
                            let mut s = autostart_state_for_menu.lock().unwrap();
                            let new_state = !*s;
                            let res = if new_state {
                                enable_autostart_impl()
                            } else {
                                disable_autostart_impl()
                            };
                            if res.is_ok() {
                                *s = new_state;
                                // NOTE: dynamic menu update via tray handle is not performed here because
                                // the current Tauri features in Cargo.toml don't expose a tray handle method.
                                // If you enable the appropriate feature that provides TrayHandle::set_menu(),
                                // you can uncomment and use that call to update the tray menu label dynamically.

                                // notify frontend (optional)
                                if let Some(win) = app.get_webview_window("main") {
                                    let event_js = format!(
                                        "window.dispatchEvent(new CustomEvent('autostart-changed', {{ detail: {{ enabled: {} }} }}));",
                                        if new_state { "true" } else { "false" }
                                    );
                                    let _ = win.eval(&event_js);
                                }
                            } else {
                                if let Some(win) = app.get_webview_window("main") {
                                    let err_msg = js_escape_single_quote(&format!("Autostart toggle failed: {:?}", res.err()));
                                    let notify_js = format!(
                                        "window.dispatchEvent(new CustomEvent('autostart-error', {{ detail: {{ message: '{}' }} }}));",
                                        err_msg
                                    );
                                    let _ = win.eval(&notify_js);
                                }
                            }
                        }
                        "show" => {
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.unminimize();
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                });

            if let Some(ic) = app.default_window_icon() {
                tray_builder = tray_builder.icon(ic.clone());
            }

            tray_builder.build(app)?;

            // Setăm titlul ferestrei principale din tauri.conf.json + versiune
            if let Some(win) = app.get_webview_window("main") {
                let title = format!("{} {}", product_name, version_label);
                let _ = win.set_title(&title);
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(context)
        .expect("Eroare la rularea aplicației");
}
