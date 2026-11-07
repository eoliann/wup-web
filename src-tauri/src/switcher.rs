use tauri::{
    webview::{PageLoadEvent, WebviewBuilder},
    AppHandle, Manager, PhysicalPosition, PhysicalSize, Rect, Webview, WebviewUrl, Window,
};

const LABEL: &str = "account-switcher";
const HEIGHT: f64 = 50.0;

pub fn bounds(window: &Window, toolbar: bool) -> tauri::Result<Rect> {
    let size = window.inner_size()?;
    let bar = ((HEIGHT * window.scale_factor()?).round() as u32).min(size.height);
    let (top, height) = if toolbar {
        (0, bar)
    } else {
        (bar, size.height - bar)
    };
    Ok(Rect {
        position: PhysicalPosition::new(0, top as i32).into(),
        size: PhysicalSize::new(size.width.max(1), height.max(1)).into(),
    })
}

pub fn create(window: &Window) -> tauri::Result<()> {
    let builder = WebviewBuilder::new(LABEL, WebviewUrl::App("index.html".into()))
        .on_navigation(|url| {
            matches!(
                (url.scheme(), url.host_str()),
                ("http" | "https", Some("tauri.localhost")) | ("tauri", Some("localhost"))
            ) && matches!(url.path(), "/" | "/index.html")
                && url.query().is_none()
        })
        .on_page_load(|view, event| {
            if event.event() == PageLoadEvent::Finished {
                sync(view.app_handle());
            }
        });
    let bounds = bounds(window, true)?;
    window.add_child(builder, bounds.position, bounds.size)?;
    Ok(())
}

pub fn resize(app: &AppHandle) {
    let Some(window) = app.get_window("main") else {
        return;
    };
    if let (Some(view), Ok(bounds)) = (app.get_webview(LABEL), bounds(&window, true)) {
        let _ = view.set_bounds(bounds);
    }
    if let Ok(bounds) = bounds(&window, false) {
        crate::accounts::resize(app, bounds);
    }
}

pub fn sync(app: &AppHandle) {
    if let Some(view) = app.get_webview(LABEL) {
        let (index, counts) = crate::accounts::toolbar_state(app);
        let _ = view.eval(format!(
            "window.showAccounts?.({index}, {})",
            serde_json::json!(counts)
        ));
    }
}

#[tauri::command]
pub async fn select_account(view: Webview, index: usize) -> Result<(), String> {
    if view.label() != LABEL || index >= crate::accounts::COUNT {
        return Err("Account selection is unavailable.".into());
    }
    let app = view.app_handle().clone();
    let handle = app.clone();
    app.run_on_main_thread(move || {
        if crate::accounts::select(&handle, index).is_err() {
            let _ = view.eval("window.switchFailed?.()");
        }
    })
    .map_err(|error| error.to_string())
}
