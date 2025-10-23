#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::{Arc, Mutex};
use std::{fs, process::Command, path::Path};

use tauri:: {
    generate_context, Manager, Wry,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIcon, TrayIconEvent, MouseButton, MouseButtonState},
    WindowEvent,
};

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

const APP_NAME: &str = env!("CARGO_PKG_NAME"); // folosește un nume stabil dacă vrei

/// Escape simplu pentru a pune un text într-un literal JS single-quoted.
fn js_escape_single_quote(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('\'', "\\'")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
}

//
// Autostart checks / impls
//
#[cfg(windows)]
fn check_autostart() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
        Ok(run_key) => run_key.get_value::<String, &str>(APP_NAME).is_ok(),
        Err(_) => false,
    }
}

#[cfg(not(windows))]
fn check_autostart() -> bool { false }

//
// Helpers to create / remove shortcut in Startup folder (best-effort).
//
#[cfg(windows)]
fn create_startup_shortcut(exe_path: &str) -> Result<(), String> {
    let lnk_name = format!("{}.lnk", APP_NAME);
    let vbs = {
        format!(r#"Set WshShell = CreateObject("WScript.Shell")
Set lnk = WshShell.CreateShortcut(WshShell.SpecialFolders("Startup") & "\{lnk}")
lnk.TargetPath = "{target}"
lnk.WorkingDirectory = "{wd}"
lnk.IconLocation = "{target}"
lnk.Save
"#,
            lnk = lnk_name,
            target = exe_path.replace('"', ""),
            wd = Path::new(exe_path).parent().and_then(|p| p.to_str()).unwrap_or(".")
        )
    };

    let mut tmp = std::env::temp_dir();
    tmp.push(format!("create_startup_{}.vbs", APP_NAME));
    fs::write(&tmp, vbs.as_bytes()).map_err(|e| format!("write vbs failed: {}", e))?;

    let output = Command::new("cscript")
        .arg("//B")
        .arg("//NoLogo")
        .arg(tmp.to_str().ok_or("tmp path invalid")?)
        .output()
        .map_err(|e| format!("spawn cscript failed: {}", e))?;

    let _ = fs::remove_file(&tmp);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("cscript failed: {}", stderr));
    }

    let startup_lnk = std::env::var("APPDATA")
        .map_err(|e| format!("APPDATA missing: {}", e))?
        + r"\Microsoft\Windows\Start Menu\Programs\Startup\";
    let lnk_path = Path::new(&startup_lnk).join(format!("{}.lnk", APP_NAME));
    if !lnk_path.exists() {
        return Err(format!("shortcut not created at {}", lnk_path.display()));
    }

    Ok(())
}
#[cfg(not(windows))]
fn create_startup_shortcut(_exe_path: &str) -> Result<(), String> {
    Err("create_startup_shortcut: not implemented on this platform".into())
}

#[cfg(windows)]
fn remove_startup_shortcut() -> Result<(), String> {
    let startup_lnk = std::env::var("APPDATA")
        .map_err(|e| format!("APPDATA missing: {}", e))?
        + r"\Microsoft\Windows\Start Menu\Programs\Startup\";
    let lnk_path = Path::new(&startup_lnk).join(format!("{}.lnk", APP_NAME));
    if lnk_path.exists() {
        fs::remove_file(&lnk_path).map_err(|e| format!("remove lnk failed: {}", e))?;
    }
    Ok(())
}
#[cfg(not(windows))]
fn remove_startup_shortcut() -> Result<(), String> {
    Err("remove_startup_shortcut: not implemented on this platform".into())
}

//
// Registry + shortcut implementations
//
#[cfg(windows)]
fn enable_autostart_impl() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe error: {}", e))?;
    let exe_str = exe.to_str().ok_or("exe path not utf8")?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run_key, _disp) = hkcu
        .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
        .map_err(|e| format!("create_subkey error: {}", e))?;
    run_key
        .set_value(APP_NAME, &format!("\"{}\"", exe_str))
        .map_err(|e| format!("reg set_value error: {}", e))?;

    // create startup shortcut (best-effort)
    match create_startup_shortcut(exe_str) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("create_startup_shortcut warning: {}", e);
            Ok(())
        }
    }
}
#[cfg(not(windows))]
fn enable_autostart_impl() -> Result<(), String> {
    Err("Autostart not implemented on this platform".into())
}

#[cfg(windows)]
fn disable_autostart_impl() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run_key) = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_WRITE,
    ) {
        let _ = run_key.delete_value(APP_NAME);
    }
    match remove_startup_shortcut() {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("remove_startup_shortcut warning: {}", e);
            Ok(())
        }
    }
}
#[cfg(not(windows))]
fn disable_autostart_impl() -> Result<(), String> {
    Err("Autostart not implemented on this platform".into())
}

//
// Tauri commands (frontend can invoke these if needed)
//
#[tauri::command]
fn is_autostart_enabled() -> Result<bool, String> { Ok(check_autostart()) }
#[tauri::command]
fn enable_autostart() -> Result<(), String> { enable_autostart_impl() }
#[tauri::command]
fn disable_autostart() -> Result<(), String> { disable_autostart_impl() }

//
// Menu builder helper (explicita generică)
//
fn build_menu_for(app: &tauri::AppHandle<Wry>, autostart_on: bool, _product: &str, version_label: &str) -> Menu<Wry> {
    let autostart_label = if autostart_on {
        "● [is ON] Disable autostart"
    } else {
        "○ [is OFF] Enable autostart"
    };

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

    let product_name: String = context
        .config()
        .product_name
        .clone()
        .unwrap_or_else(|| context.package_info().name.clone());
    let version_label = format!("{}", env!("CARGO_PKG_VERSION"));

    let initial_autostart = check_autostart();
    let autostart_state = Arc::new(Mutex::new(initial_autostart));

    // aici stocăm tray handle-ul pentru a-l putea folosi la runtime (set_menu)
    let tray_handle_shared: Arc<Mutex<Option<TrayIcon>>> = Arc::new(Mutex::new(None));

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            is_autostart_enabled,
            enable_autostart,
            disable_autostart
        ])
        .setup({
            let product_for_closure = product_name.clone();
            let version_for_closure = version_label.clone();
            let autostart_state_for_menu = autostart_state.clone();
            let tray_handle_shared_setup = tray_handle_shared.clone();

            move |app| {
                // construit meniul inițial
                let menu = build_menu_for(&app.app_handle(), initial_autostart, &product_for_closure, &version_for_closure);

                // clone pentru closure-ul on_menu_event
                let product_for_menu = product_for_closure.clone();
                let version_for_menu = version_for_closure.clone();
                let autostart_state_for_event = autostart_state_for_menu.clone();
                let tray_handle_for_event = tray_handle_shared_setup.clone();

                // construim builder-ul tray și setăm handler-ul de menu
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
                                let p = js_escape_single_quote(&product_for_menu);
                                let v = js_escape_single_quote(&version_for_menu);
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
                                // toggle state + update registry/shortcut
                                let mut s = autostart_state_for_event.lock().unwrap();
                                let new_state = !*s;
                                let res = if new_state {
                                    enable_autostart_impl()
                                } else {
                                    disable_autostart_impl()
                                };
                                if res.is_ok() {
                                    *s = new_state;

                                    // rebuild menu cu noua etichetă
                                    let new_menu = build_menu_for(&app, new_state, &product_for_menu, &version_for_menu);

                                    // actualizează meniul tray dacă avem handle-ul (apel set_menu)
                                    if let Some(tray_handle) = tray_handle_for_event.lock().unwrap().as_ref() {
                                        // TrayIcon::set_menu accepts Option<Menu<R>>
                                        // Folosim unwrap_or_else pentru a loga erorile la runtime.
                                        let _ = tray_handle.set_menu(Some(new_menu));
                                    } else {
                                        eprintln!("tray handle not stored yet; menu won't refresh immediately");
                                    }

                                    // notifica frontend
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

                // BUILD tray-ul și păstrăm handle-ul rezultat într-un Arc<Mutex<Option<TrayIcon>>>
                let tray_icon = tray_builder.build(app)?;
                {
                    let mut guard = tray_handle_shared_setup.lock().unwrap();
                    *guard = Some(tray_icon.clone());
                }

                // setăm titlul ferestrei principale
                if let Some(win) = app.get_webview_window("main") {
                    let title = format!("{} {}", product_for_closure, version_for_closure);
                    let _ = win.set_title(&title);
                }

                Ok(())
            }
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
