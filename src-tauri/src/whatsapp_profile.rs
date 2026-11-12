use crate::{account_preferences::AVATAR_SIZE, accounts};
use std::time::{Duration, Instant};
use tauri::{async_runtime::Sender, AppHandle};
use webview2_com::CallDevToolsProtocolMethodCompletedHandler;
use windows_core::{w, HSTRING, PCWSTR};

const UNAVAILABLE: &str = "Open your WhatsApp profile and try syncing the photo again.";

pub fn sync(app: &AppHandle, index: usize, reply: Sender<Result<(), String>>) {
    let Some(view) = accounts::view(app, index) else {
        let _ = reply.try_send(Err(UNAVAILABLE.into()));
        return;
    };
    if !view.url().is_ok_and(|url| accounts::trusted(&url)) {
        let _ = reply.try_send(Err(UNAVAILABLE.into()));
        return;
    }
    let app = app.clone();
    let deadline = Instant::now() + Duration::from_secs(30);
    let callback_reply = reply.clone();
    let start = view.with_webview(move |platform| {
        let params = HSTRING::from(
            serde_json::json!({
                "expression": format!("({})({AVATAR_SIZE})", include_str!("whatsapp_profile.js")),
                "awaitPromise": true,
                "returnByValue": true,
            })
            .to_string(),
        );
        let result_reply = callback_reply.clone();
        let handler = CallDevToolsProtocolMethodCompletedHandler::create(Box::new(
            move |result, response| {
                let outcome = if result.is_ok()
                    && Instant::now() < deadline
                    && accounts::selected(&app) == index
                {
                    store_photo(&app, index, &response)
                } else {
                    Err(UNAVAILABLE.into())
                };
                let _ = result_reply.try_send(outcome);
                Ok(())
            },
        ));
        // The controller and the completion callback both run on the WebView2 UI thread.
        let started = unsafe {
            platform.controller().CoreWebView2().and_then(|webview| {
                webview.CallDevToolsProtocolMethod(
                    w!("Runtime.evaluate"),
                    PCWSTR(params.as_ptr()),
                    &handler,
                )
            })
        };
        if started.is_err() {
            let _ = callback_reply.try_send(Err(UNAVAILABLE.into()));
        }
    });
    if start.is_err() {
        let _ = reply.try_send(Err(UNAVAILABLE.into()));
    }
}

fn store_photo(app: &AppHandle, index: usize, response: &str) -> Result<(), String> {
    let result: serde_json::Value = serde_json::from_str(response).map_err(|_| UNAVAILABLE)?;
    match result["result"]["value"].as_str() {
        Some("timeout") => Err("Photo selection timed out.".into()),
        Some("unreadable") => {
            Err("WhatsApp did not provide readable pixels for this photo.".into())
        }
        Some(pixels) => accounts::set_avatar(app, index, Some(pixels.to_owned())),
        None => Err(UNAVAILABLE.into()),
    }
}
