use std::{fs, io, path::Path};
use tauri::image::Image;

pub const AVATAR_SIZE: u32 = 48;
const HEX_LENGTH: usize = (AVATAR_SIZE * AVATAR_SIZE * 4 * 2) as usize;

#[derive(Clone, Default)]
pub struct Preferences {
    pub name: Option<String>,
    pub avatar: Option<String>,
}

pub fn normalize_name(name: &str) -> Result<Option<String>, &'static str> {
    let name = name.trim();
    if name.chars().count() > 40 || name.chars().any(char::is_control) {
        return Err("Use up to 40 characters for the account name.");
    }
    Ok((!name.is_empty()).then(|| name.to_owned()))
}

pub fn valid_avatar(avatar: &str) -> bool {
    avatar.len() == HEX_LENGTH && avatar.bytes().all(|byte| byte.is_ascii_hexdigit())
}

impl Preferences {
    pub fn load(path: &Path) -> Self {
        let Ok(bytes) = fs::read(path) else {
            return Self::default();
        };
        let Ok(data) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            return Self::default();
        };
        Self {
            name: data["name"]
                .as_str()
                .and_then(|name| normalize_name(name).ok().flatten()),
            avatar: data["avatar"]
                .as_str()
                .filter(|avatar| valid_avatar(avatar))
                .map(str::to_owned),
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let data = serde_json::json!({ "name": self.name, "avatar": self.avatar });
        let temp = path.with_extension("tmp");
        fs::write(&temp, data.to_string())?;
        fs::rename(temp, path)
    }

    pub fn display_name(&self, index: usize) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| format!("Account {}", index + 1))
    }

    pub fn icon(&self) -> Option<Image<'static>> {
        let avatar = self.avatar.as_ref()?;
        let rgba = (0..avatar.len())
            .step_by(2)
            .map(|offset| u8::from_str_radix(&avatar[offset..offset + 2], 16))
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        Some(Image::new_owned(rgba, AVATAR_SIZE, AVATAR_SIZE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_names_restore_defaults_and_unicode_is_supported() {
        assert_eq!(normalize_name("  "), Ok(None));
        assert_eq!(normalize_name("  Echipă  "), Ok(Some("Echipă".into())));
        assert!(normalize_name("One\nTwo").is_err());
        assert!(normalize_name(&"ă".repeat(41)).is_err());
        assert_eq!(Preferences::default().display_name(3), "Account 4");
    }

    #[test]
    fn avatar_accepts_only_fixed_size_rgba_pixels() {
        assert!(valid_avatar(&"ff".repeat(HEX_LENGTH / 2)));
        assert!(!valid_avatar(&"gg".repeat(HEX_LENGTH / 2)));
        assert!(!valid_avatar("ff"));
    }
}
