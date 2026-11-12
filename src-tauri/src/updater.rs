use semver::Version;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{self, Read},
    os::windows::{ffi::OsStrExt, process::CommandExt},
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager, Url};
use windows::{
    core::{w, PCWSTR},
    Win32::UI::Shell::ShellExecuteW,
    Win32::UI::WindowsAndMessaging::{
        MessageBoxW, IDYES, MB_ICONINFORMATION, MB_OK, MB_YESNO, MESSAGEBOX_STYLE, SW_SHOWNORMAL,
    },
};

const API: &str = "https://api.github.com/repos/eoliann/wup-web/releases/latest";
const RELEASES: &str = "https://github.com/eoliann/wup-web/releases/latest";
const ASSETS: &str = "/eoliann/wup-web/releases/download/";
const HIDDEN: u32 = 0x08000000;
static BUSY: AtomicBool = AtomicBool::new(false);

struct Asset {
    url: Url,
    digest: String,
}

fn parse_version(tag: &str) -> Option<Version> {
    Version::parse(tag.strip_prefix('v').unwrap_or(tag))
        .ok()
        .filter(|version| version.pre.is_empty())
}

fn find_asset(release: &Value) -> Option<Asset> {
    release["assets"].as_array()?.iter().find_map(|asset| {
        if asset["name"].as_str()? != "Wup_Web_Installer.exe" {
            return None;
        }
        let url = Url::parse(asset["browser_download_url"].as_str()?).ok()?;
        if url.scheme() != "https"
            || url.host_str() != Some("github.com")
            || url.port().is_some()
            || !url.username().is_empty()
            || url.password().is_some()
            || !url.path().starts_with(ASSETS)
        {
            return None;
        }
        let digest = asset["digest"].as_str()?.strip_prefix("sha256:")?;
        if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        Some(Asset {
            url,
            digest: digest.to_ascii_lowercase(),
        })
    })
}

fn curl() -> Command {
    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into());
    let mut command = Command::new(PathBuf::from(root).join(r"System32\curl.exe"));
    command.creation_flags(HIDDEN).args([
        "--disable",
        "--fail",
        "--silent",
        "--show-error",
        "--location",
        "--proto",
        "=https",
        "--proto-redir",
        "=https",
        "--max-redirs",
        "5",
        "--connect-timeout",
        "10",
        "--user-agent",
        "Wup-Web-Updater",
    ]);
    command
}

fn latest() -> Result<Option<Value>, String> {
    let output = curl()
        .args([
            "--max-time",
            "20",
            "--max-filesize",
            "1048576",
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "Cache-Control: no-cache",
            "--write-out",
            "\n%{http_code}",
            API,
        ])
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() && output.stdout.ends_with(b"\n404") {
        return Ok(None);
    }
    if !output.status.success() {
        return Err("GitHub could not be reached. Try again later.".into());
    }
    let (json, _) = output.stdout.split_at(
        output
            .stdout
            .len()
            .checked_sub(4)
            .ok_or("Invalid GitHub response")?,
    );
    serde_json::from_slice(json)
        .map(Some)
        .map_err(|error| error.to_string())
}

fn checksum(path: &Path) -> io::Result<String> {
    let mut input = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 32 * 1024];
    loop {
        let size = input.read(&mut buffer)?;
        if size == 0 {
            return Ok(format!("{:x}", hash.finalize()));
        }
        hash.update(&buffer[..size]);
    }
}

fn download(asset: &Asset, directory: &Path) -> Result<PathBuf, String> {
    let path = directory.join("update.exe");
    let status = curl()
        .args([
            "--max-time",
            "300",
            "--max-filesize",
            "268435456",
            "--output",
        ])
        .arg(&path)
        .arg(asset.url.as_str())
        .status()
        .map_err(|error| error.to_string())?;
    if !status.success() {
        return Err("Download failed.".into());
    }
    if checksum(&path).map_err(|error| error.to_string())? != asset.digest {
        return Err("The downloaded file does not match GitHub’s SHA-256 digest.".into());
    }
    Ok(path)
}

fn launch_update(path: &Path) -> io::Result<()> {
    let installer: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    // The Windows shell handles elevation for the interactive installer.
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(installer.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize > 32 {
        Ok(())
    } else {
        Err(io::Error::other("Windows could not start the installer."))
    }
}

fn dialog(app: &AppHandle, message: String, style: MESSAGEBOX_STYLE) -> bool {
    let (send, receive) = mpsc::sync_channel(1);
    let handle = app.clone();
    if app
        .run_on_main_thread(move || {
            let message: Vec<u16> = message.encode_utf16().chain([0]).collect();
            let title: Vec<u16> = "Wup Web Update".encode_utf16().chain([0]).collect();
            let owner = handle
                .get_window("main")
                .and_then(|window| window.hwnd().ok());
            // Both UTF-16 buffers remain live until the modal dialog returns.
            let result = unsafe {
                MessageBoxW(
                    owner,
                    PCWSTR(message.as_ptr()),
                    PCWSTR(title.as_ptr()),
                    style,
                )
            };
            let _ = send.send(result == IDYES);
        })
        .is_err()
    {
        return false;
    }
    receive.recv().unwrap_or(false)
}

fn run(app: &AppHandle, manual: bool) -> Result<(), String> {
    let Some(release) = latest()? else {
        if manual {
            dialog(
                app,
                "GitHub has no published release.".into(),
                MB_OK | MB_ICONINFORMATION,
            );
        }
        return Ok(());
    };
    let version = release["tag_name"]
        .as_str()
        .and_then(parse_version)
        .ok_or("Invalid release version")?;
    let current = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    if version <= current {
        if manual {
            dialog(
                app,
                format!("Wup Web {current} is up to date."),
                MB_OK | MB_ICONINFORMATION,
            );
        }
        return Ok(());
    }

    let Some(asset) = find_asset(&release) else {
        if dialog(app, format!("Wup Web {version} is available. A matching verified file is unavailable. Open GitHub Releases?"), MB_YESNO | MB_ICONINFORMATION) {
            Command::new("explorer.exe").arg(RELEASES).spawn().map_err(|error| error.to_string())?;
        }
        return Ok(());
    };
    if !dialog(app, format!("Wup Web {version} is available (installed: {current}). Download, verify and apply the update?"), MB_YESNO | MB_ICONINFORMATION) { return Ok(()); }

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("wup-update-{}-{nonce}", std::process::id()));
    fs::create_dir(&directory).map_err(|error| error.to_string())?;
    if let Err(error) = download(&asset, &directory)
        .and_then(|path| launch_update(&path).map_err(|error| error.to_string()))
    {
        let _ = fs::remove_file(directory.join("update.exe"));
        dialog(app, error, MB_OK | MB_ICONINFORMATION);
        return Ok(());
    }
    app.exit(0);
    Ok(())
}

pub fn check(app: &AppHandle, manual: bool) {
    if BUSY.swap(true, Ordering::AcqRel) {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        if let Err(error) = run(&app, manual) {
            if manual {
                dialog(&app, error, MB_OK | MB_ICONINFORMATION);
            }
        }
        BUSY.store(false, Ordering::Release);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    #[ignore = "downloads the current public GitHub installer"]
    fn published_github_assets_download_and_verify() {
        let release = latest()
            .expect("GitHub check failed")
            .expect("No published release");
        assert!(release["tag_name"]
            .as_str()
            .and_then(parse_version)
            .is_some());
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("wup-release-check-{nonce}"));
        fs::create_dir(&directory).unwrap();
        let asset = find_asset(&release).expect("No verified installer asset");
        let path = download(&asset, &directory)
            .expect("Installer download or SHA-256 verification failed");
        fs::remove_file(path).unwrap();
        fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn stable_versions_are_compared_numerically() {
        assert!(parse_version("v2.0.10").unwrap() > parse_version("2.0.9").unwrap());
        assert!(parse_version("v2.0.1-beta.1").is_none());
        assert!(parse_version("unversioned").is_none());
    }

    #[test]
    fn requires_installer_name_repository_and_digest() {
        let release = |url: &str, digest: String| {
            json!({"assets": [{
                "name": "Wup_Web_Installer.exe", "browser_download_url": url, "digest": digest
            }]})
        };
        let url =
            "https://github.com/eoliann/wup-web/releases/download/v2.0.1/Wup_Web_Installer.exe";
        let valid = release(url, format!("sha256:{}", "0".repeat(64)));
        assert!(find_asset(&valid).is_some());
        let mut portable = valid.clone();
        portable["assets"][0]["name"] = json!("wup-web.exe");
        assert!(find_asset(&portable).is_none());
        assert!(find_asset(&release(url, "sha256:no".into())).is_none());
        assert!(find_asset(&release(
            "https://example.com/update.exe",
            format!("sha256:{}", "0".repeat(64))
        ),)
        .is_none());
    }
}
