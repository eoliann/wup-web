#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{
    generate_context, Manager,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
    WindowEvent,
};

/// Escape simplu pentru a pune un text într-un literal JS single-quoted.
/// Acoperă backslash, single quote și newline/carriage returns.
fn js_escape_single_quote(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('\'', "\\'")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
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
    let version_label = format!("v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .setup(move |app| {
            // Construim item-urile meniului (version este clicabil)
            let version_item = MenuItem::with_id(
                app,
                "version",
                &format!("Version: {}", env!("CARGO_PKG_VERSION")),
                true, // enabled (clicabil)
                None::<&str>,
            )?;

            // Separator "simulat" — dacă faci upgrade la alt API Tauri putem folosi separator nativ
            let sep_item = MenuItem::with_id(app, "sep", "──────────", false, None::<&str>)?;

            let show = MenuItem::with_id(app, "show", "Open", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&version_item, &sep_item, &show, &quit])?;

            // clonate pentru closure-urile interne
            let product_for_closure = product_name.clone();
            let version_for_closure = version_label.clone();

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
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "version" => {
                        // Cream textul changelog (placeholder). Înlocuiește cu conținut real dacă vrei.
                        let changelog_text = "Placeholder changelog — înlocuiește cu textul real din CHANGELOG.md";

                        // Escapăm valorile pentru a fi safe într-un literal JS single-quoted
                        let p = js_escape_single_quote(&product_for_closure);
                        let v = js_escape_single_quote(&version_for_closure);
                        let c = js_escape_single_quote(changelog_text);

                        // Construim cod JS care va declanșa un CustomEvent 'show-changelog'
                        // Detaliul event-ului conține obiectul { product, version, changelog }
                        let js = format!(
                            "window.dispatchEvent(new CustomEvent('show-changelog', {{ detail: {{ product: '{}', version: '{}', changelog: '{}' }} }}));",
                            p, v, c
                        );

                        if let Some(win) = app.get_webview_window("main") {
                            // Eval execută cod JS în webview (fără a folosi serde).
                            let _ = win.eval(&js);
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
