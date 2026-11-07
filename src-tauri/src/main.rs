#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{fs, path::Path, process::Command};

mod accounts;
mod badges;
mod desktop;
mod switcher;
#[cfg(windows)]
mod updater;

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

const APP_NAME: &str = env!("CARGO_PKG_NAME"); // folosește un nume stabil dacă vrei

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
fn check_autostart() -> bool {
    false
}

//
// Helpers to create / remove shortcut in Startup folder (best-effort).
//
#[cfg(windows)]
fn create_startup_shortcut(exe_path: &str) -> Result<(), String> {
    let lnk_name = format!("{}.lnk", APP_NAME);
    let vbs = {
        format!(
            r#"Set WshShell = CreateObject("WScript.Shell")
Set lnk = WshShell.CreateShortcut(WshShell.SpecialFolders("Startup") & "\{lnk}")
lnk.TargetPath = "{target}"
lnk.WorkingDirectory = "{wd}"
lnk.IconLocation = "{target}"
lnk.Save
"#,
            lnk = lnk_name,
            target = exe_path.replace('"', ""),
            wd = Path::new(exe_path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or(".")
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

    let startup_lnk = std::env::var("APPDATA").map_err(|e| format!("APPDATA missing: {}", e))?
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
    let startup_lnk = std::env::var("APPDATA").map_err(|e| format!("APPDATA missing: {}", e))?
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

fn main() {
    #[cfg(windows)]
    {
        use windows::{
            core::w,
            Win32::{
                Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
                System::Threading::CreateMutexW,
            },
        };
        // Windows closes the handle when the application exits.
        let instance = unsafe { CreateMutexW(None, false, w!("Local\\eoliann.wup-web")) };
        if instance.is_ok() && unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            return;
        }
    }
    desktop::run();
}
