use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WindowBuilder, WindowEvent,
};

use crate::{accounts, check_autostart, disable_autostart_impl, enable_autostart_impl};

fn menu(app: &AppHandle, tray: bool) -> tauri::Result<(Menu<tauri::Wry>, MenuItem<tauri::Wry>)> {
    let menu = Menu::new(app)?;
    accounts::append_menu(app, &menu)?;
    if tray {
        menu.append(&MenuItem::with_id(app, "show", "Open", true, None::<&str>)?)?;
    }
    let autostart = MenuItem::with_id(
        app,
        "autostart",
        autostart_label(check_autostart()),
        true,
        None::<&str>,
    )?;
    menu.append(&autostart)?;
    #[cfg(windows)]
    menu.append(&MenuItem::with_id(
        app,
        "updates",
        "Check for updates",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?)?;
    Ok((menu, autostart))
}

fn autostart_label(enabled: bool) -> &'static str {
    if enabled {
        "Disable autostart"
    } else {
        "Enable autostart"
    }
}

pub fn run() {
    let autostart_items = Arc::new(Mutex::new(Vec::<MenuItem<tauri::Wry>>::new()));
    let menu_items = autostart_items.clone();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![crate::switcher::select_account])
        .setup(move |app| {
            let handle = app.handle();
            let (window_menu, window_autostart) = menu(handle, false)?;
            let (tray_menu, tray_autostart) = menu(handle, true)?;
            menu_items
                .lock()
                .unwrap()
                .extend([window_autostart, tray_autostart]);
            let window = WindowBuilder::new(app, "main")
                .inner_size(1000.0, 700.0)
                .min_inner_size(320.0, 250.0)
                .visible(false)
                .menu(window_menu)
                .build()?;
            let mut tray = TrayIconBuilder::with_id("wup")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .tooltip("Wup Web")
                .on_tray_icon_event(|tray, event| {
                    if matches!(
                        event,
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        }
                    ) {
                        let _ = accounts::show(tray.app_handle());
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;

            let root = app.path().app_local_data_dir()?;
            crate::switcher::create(&window)?;
            accounts::restore_profiles(handle, &root)?;
            accounts::select(handle, 0)?;
            let handle = handle.clone();
            window.on_window_event(move |event| match event {
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    if let Some(window) = handle.get_window("main") {
                        let _ = window.hide();
                    }
                }
                WindowEvent::Focused(true) => accounts::refresh_badges(&handle),
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                    crate::switcher::resize(&handle);
                }
                _ => (),
            });
            #[cfg(windows)]
            if !std::env::args().any(|arg| arg == "--no-update-check") {
                crate::updater::check(app.handle(), false);
            }
            Ok(())
        })
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            let result = match id {
                "show" => accounts::show(app),
                "autostart" => {
                    let result = if check_autostart() {
                        disable_autostart_impl()
                    } else {
                        enable_autostart_impl()
                    };
                    if let Err(error) = result {
                        eprintln!("Autostart: {error}");
                    }
                    for item in autostart_items.lock().unwrap().iter() {
                        let _ = item.set_text(autostart_label(check_autostart()));
                    }
                    Ok(())
                }
                #[cfg(windows)]
                "updates" => {
                    crate::updater::check(app, true);
                    Ok(())
                }
                "quit" => {
                    app.exit(0);
                    Ok(())
                }
                _ => match accounts::index(id) {
                    Some(index) => accounts::select(app, index),
                    None => Ok(()),
                },
            };
            if let Err(error) = result {
                eprintln!("Wup Web menu action: {error}");
            }
        })
        .run(tauri::generate_context!())
        .expect("Wup Web startup failed");
}
