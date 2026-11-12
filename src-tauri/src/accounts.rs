use crate::account_preferences::{normalize_name, valid_avatar, Preferences};
use std::{fs, path::PathBuf, sync::Mutex};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, Submenu},
    webview::WebviewBuilder,
    AppHandle, Manager, Url, WebviewUrl,
};

const LABELS: [&str; 4] = ["account-1", "account-2", "account-3", "account-4"];
pub const COUNT: usize = LABELS.len();
const WHATSAPP: &str = "https://web.whatsapp.com/";

#[derive(Default)]
struct State {
    selected: usize,
    counts: [Option<u32>; LABELS.len()],
    preferences: [Preferences; COUNT],
}

pub struct Accounts {
    root: PathBuf,
    state: Mutex<State>,
    menu_items: Mutex<Vec<(usize, MenuItem<tauri::Wry>)>>,
}

impl Accounts {
    pub fn new(root: PathBuf) -> Self {
        let preferences = std::array::from_fn(|index| {
            Preferences::load(&root.join(format!("{}.json", LABELS[index])))
        });
        Self {
            root,
            state: Mutex::new(State {
                preferences,
                ..State::default()
            }),
            menu_items: Mutex::default(),
        }
    }

    pub fn profile_path(&self, index: usize) -> PathBuf {
        if index == 0 {
            self.root.clone()
        } else {
            self.root.join("profiles").join(LABELS[index])
        }
    }
}

pub fn index(id: &str) -> Option<usize> {
    LABELS.iter().position(|label| *label == id)
}

pub fn append_menu(app: &AppHandle, menu: &Menu<tauri::Wry>) -> tauri::Result<()> {
    let accounts = app.state::<Accounts>();
    let submenu = Submenu::new(app, "Accounts", true)?;
    for (index, label) in LABELS.iter().enumerate() {
        let item = MenuItem::with_id(
            app,
            *label,
            menu_name(&accounts.state.lock().unwrap().preferences[index], index),
            true,
            None::<&str>,
        )?;
        submenu.append(&item)?;
        accounts.menu_items.lock().unwrap().push((index, item));
    }
    menu.append(&submenu)
}

fn menu_name(preferences: &Preferences, index: usize) -> String {
    preferences.display_name(index).replace('&', "&&")
}

pub fn trusted(url: &Url) -> bool {
    url.scheme() == "https" && url.host_str() == Some("web.whatsapp.com") && url.port().is_none()
}

fn unread_candidate(title: &str) -> Option<u32> {
    if title == "WhatsApp" {
        return Some(0);
    }
    let (number, app) = title.strip_prefix('(')?.split_once(") ")?;
    if app != "WhatsApp" || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    number.parse().ok()
}

fn open_view(app: &AppHandle, index: usize, path: PathBuf) -> tauri::Result<()> {
    let window = app.get_window("main").expect("main window");
    let builder = WebviewBuilder::new(
        LABELS[index],
        WebviewUrl::External("about:blank".parse().unwrap()),
    )
    .data_directory(path)
    // WebView2 must deliver native file drops to the page as HTML5 events.
    .disable_drag_drop_handler()
    .on_navigation(|url| url.as_str() == "about:blank" || trusted(url))
    .on_document_title_changed(move |view, title| {
        let app = view.app_handle();
        let accounts = app.state::<Accounts>();
        let count = unread_candidate(&title);
        let mut state = accounts.state.lock().unwrap();
        let changed = state.counts[index] != count;
        state.counts[index] = count;
        let refresh = changed && state.selected == index;
        drop(state);
        if changed {
            crate::switcher::sync(app);
        }
        if refresh {
            crate::badges::refresh(app, count);
        }
    });
    let bounds = crate::switcher::bounds(&window, false)?;
    let view = window.add_child(builder, bounds.position, bounds.size)?;
    view.hide()?;
    #[cfg(windows)]
    view.with_webview(|platform| {
        if let Err(error) = permit_web_notifications(platform.controller()) {
            eprintln!("WebView2 notification permissions: {error}");
        }
    })?;
    view.navigate(WHATSAPP.parse().unwrap())
}

#[cfg(windows)]
fn permit_web_notifications(
    controller: webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller,
) -> windows_core::Result<()> {
    use webview2_com::{
        take_pwstr, Microsoft::Web::WebView2::Win32::*, PermissionRequestedEventHandler,
    };
    use windows_core::PWSTR;
    // The WebView2 UI thread owns the controller and the callback registration.
    unsafe {
        let handler = PermissionRequestedEventHandler::create(Box::new(|_, args| {
            let Some(args) = args else {
                return Ok(());
            };
            let mut kind = COREWEBVIEW2_PERMISSION_KIND::default();
            args.PermissionKind(&mut kind)?;
            if kind == COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS {
                let mut uri = PWSTR::null();
                args.Uri(&mut uri)?;
                if Url::parse(&take_pwstr(uri)).is_ok_and(|url| trusted(&url)) {
                    args.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW)?;
                }
            }
            Ok(())
        }));
        controller
            .CoreWebView2()?
            .add_PermissionRequested(&handler, &mut 0)
    }
}

pub fn restore_profiles(app: &AppHandle) -> tauri::Result<()> {
    let accounts = app.state::<Accounts>();
    open_view(app, 0, accounts.profile_path(0))?;
    for index in 1..LABELS.len() {
        let path = accounts.profile_path(index);
        if path.is_dir() {
            if let Err(error) = open_view(app, index, path) {
                eprintln!("Could not reopen account {}: {error}", index + 1);
            }
        }
    }
    Ok(())
}

pub fn select(app: &AppHandle, index: usize) -> tauri::Result<()> {
    let accounts = app.state::<Accounts>();
    if app.get_webview(LABELS[index]).is_none() {
        open_view(app, index, accounts.profile_path(index))?;
    }
    let previous = accounts.state.lock().unwrap().selected;
    let view = app.get_webview(LABELS[index]).expect("account webview");
    view.show()?;
    if previous != index {
        if let Some(previous) = app.get_webview(LABELS[previous]) {
            previous.hide()?;
        }
    }
    accounts.state.lock().unwrap().selected = index;
    crate::switcher::sync(app);
    show(app)
}

pub fn selected(app: &AppHandle) -> usize {
    app.try_state::<Accounts>()
        .map_or(0, |accounts| accounts.state.lock().unwrap().selected)
}

pub fn toolbar_state(app: &AppHandle) -> (usize, [Option<u32>; COUNT], serde_json::Value) {
    let accounts = app.state::<Accounts>();
    let state = accounts.state.lock().unwrap();
    let preferences: Vec<_> = state
        .preferences
        .iter()
        .enumerate()
        .map(|(index, preferences)| {
            serde_json::json!({
                "label": preferences.display_name(index),
                "name": preferences.name,
                "avatar": preferences.avatar,
            })
        })
        .collect();
    (state.selected, state.counts, serde_json::json!(preferences))
}

#[cfg(windows)]
pub fn view(app: &AppHandle, index: usize) -> Option<tauri::Webview> {
    app.get_webview(LABELS[index])
}

pub fn selected_icon(app: &AppHandle) -> Option<Image<'static>> {
    let accounts = app.state::<Accounts>();
    let state = accounts.state.lock().unwrap();
    state.preferences[state.selected].icon()
}

pub fn rename(app: &AppHandle, index: usize, name: &str) -> Result<(), String> {
    let name = normalize_name(name)?;
    update_preferences(app, index, |preferences| preferences.name = name)
}

pub fn set_avatar(app: &AppHandle, index: usize, avatar: Option<String>) -> Result<(), String> {
    if avatar
        .as_deref()
        .is_some_and(|avatar| !valid_avatar(avatar))
    {
        return Err("Could not read the profile photo.".into());
    }
    update_preferences(app, index, |preferences| preferences.avatar = avatar)
}

fn update_preferences(
    app: &AppHandle,
    index: usize,
    update: impl FnOnce(&mut Preferences),
) -> Result<(), String> {
    let accounts = app.state::<Accounts>();
    let mut state = accounts.state.lock().unwrap();
    let mut preferences = state.preferences[index].clone();
    update(&mut preferences);
    let path = accounts.root.join(format!("{}.json", LABELS[index]));
    fs::create_dir_all(&accounts.root)
        .and_then(|()| preferences.save(&path))
        .map_err(|_| "Could not save account settings.")?;
    let label = menu_name(&preferences, index);
    state.preferences[index] = preferences;
    let selected = state.selected == index;
    drop(state);
    for (item_index, item) in accounts.menu_items.lock().unwrap().iter() {
        if *item_index == index {
            let _ = item.set_text(&label);
        }
    }
    crate::switcher::sync(app);
    if selected {
        let _ = refresh_title(app);
        refresh_badges(app);
    }
    Ok(())
}

pub fn resize(app: &AppHandle, bounds: tauri::Rect) {
    for label in LABELS {
        if let Some(view) = app.get_webview(label) {
            let _ = view.set_bounds(bounds);
        }
    }
}

pub fn refresh_badges(app: &AppHandle) {
    if let Some(accounts) = app.try_state::<Accounts>() {
        let state = accounts.state.lock().unwrap();
        let count = state.counts[state.selected];
        drop(state);
        crate::badges::refresh(app, count);
    }
}

fn refresh_title(app: &AppHandle) -> tauri::Result<()> {
    let accounts = app.state::<Accounts>();
    let state = accounts.state.lock().unwrap();
    let title = state.preferences[state.selected].display_name(state.selected);
    drop(state);
    app.get_window("main")
        .expect("main window")
        .set_title(&format!("Wup Web {} • {title}", env!("CARGO_PKG_VERSION"),))
}

pub fn show(app: &AppHandle) -> tauri::Result<()> {
    let index = selected(app);
    let window = app.get_window("main").expect("main window");
    refresh_title(app)?;
    window.show()?;
    window.unminimize()?;
    window.set_focus()?;
    if let Some(view) = app.get_webview(LABELS[index]) {
        view.set_focus()?;
    }
    refresh_badges(app);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_whatsapp_title_format() {
        assert_eq!(unread_candidate("(3) WhatsApp"), Some(3));
        assert_eq!(unread_candidate("WhatsApp"), Some(0));
        assert_eq!(unread_candidate("(99+) WhatsApp"), None);
        assert_eq!(unread_candidate("(3) Another page"), None);
        assert_eq!(unread_candidate("Contact 3"), None);
    }

    #[test]
    fn account_ids_are_limited_to_four() {
        assert_eq!(index("account-4"), Some(3));
        assert_eq!(index("account-5"), None);
    }
}
