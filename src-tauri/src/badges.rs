use tauri::{image::Image, AppHandle, Manager};

const DIGITS: [[u8; 5]; 10] = [
    [7, 5, 5, 5, 7],
    [2, 6, 2, 2, 7],
    [7, 1, 7, 4, 7],
    [7, 1, 7, 1, 7],
    [5, 5, 7, 1, 1],
    [7, 4, 7, 1, 7],
    [7, 4, 7, 5, 7],
    [7, 1, 1, 1, 1],
    [7, 5, 7, 5, 7],
    [7, 5, 7, 1, 7],
];

fn image(size: usize, base: Option<&Image<'_>>, label: &str) -> Image<'static> {
    let mut rgba = vec![0; size * size * 4];
    if let Some(base) = base {
        let width = base.width() as usize;
        let height = base.height() as usize;
        for y in 0..size {
            for x in 0..size {
                let source = (y * height / size * width + x * width / size) * 4;
                let target = (y * size + x) * 4;
                rgba[target..target + 4].copy_from_slice(&base.rgba()[source..source + 4]);
            }
        }
    }
    let scale = size / 16;
    let left = (size - (label.len() * 4 - 1) * scale) / 2;
    let top = if base.is_some() {
        size - 6 * scale
    } else {
        5 * scale
    };
    for y in top - scale..top + 6 * scale {
        for x in 0..size {
            let pixel = (y * size + x) * 4;
            rgba[pixel..pixel + 4].copy_from_slice(&[0, 66, 46, 255]);
        }
    }
    for (index, digit) in label.bytes().enumerate() {
        let rows = if digit == b'+' {
            [0, 2, 7, 2, 0]
        } else {
            DIGITS[(digit - b'0') as usize]
        };
        for (y, row) in rows.into_iter().enumerate() {
            for x in 0..3 {
                if row & (4 >> x) == 0 {
                    continue;
                }
                for dy in 0..scale {
                    for dx in 0..scale {
                        let pixel =
                            ((top + y * scale + dy) * size + left + (index * 4 + x) * scale + dx)
                                * 4;
                        rgba[pixel..pixel + 4].copy_from_slice(&[255, 244, 181, 255]);
                    }
                }
            }
        }
    }
    Image::new_owned(rgba, size as u32, size as u32)
}

pub fn refresh(app: &AppHandle, count: Option<u32>) {
    let label = count.filter(|&count| count > 0).map(|count| {
        if count > 9999 {
            "999+".into()
        } else {
            count.to_string()
        }
    });
    let avatar = crate::accounts::selected_icon(app);
    let base = avatar.as_ref().or_else(|| app.default_window_icon());
    if let Some(tray) = app.tray_by_id("wup") {
        let icon = label
            .as_deref()
            .map(|label| image(32, base, label))
            .or_else(|| base.cloned());
        let _ = tray.set_icon(icon);
        let tooltip = count.map_or_else(
            || "Wup Web".into(),
            |count| format!("Wup Web • {count} unread"),
        );
        let _ = tray.set_tooltip(Some(tooltip));
    }
    #[cfg(windows)]
    if let Some(window) = app.get_window("main") {
        if let Some(icon) = base {
            let _ = window.set_icon(icon.clone());
        }
        let _ = window.set_overlay_icon(label.as_deref().map(|label| image(16, None, label)));
    }
}
