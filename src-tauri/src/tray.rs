use chrono::{Local, Utc};
use tauri::image::Image;
use tauri::menu::{Menu, MenuBuilder, MenuEvent, MenuItemBuilder, SubmenuBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Wry};

use crate::i18n::{duration_since, s, sites_offline};
use crate::state::{AppState, Level, StatusView};
use crate::ui;

const TRAY_ID: &str = "main";
const ICON_SIZE: u32 = 32;
const MAX_LABEL_CHARS: usize = 60;

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let view = app.state::<AppState>().lock().view(Utc::now());
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon_for(view.level))
        .tooltip("UptimeWidget")
        .menu(&build_menu(app, &view)?)
        .show_menu_on_left_click(true)
        .on_menu_event(on_menu_event)
        .build(app)?;
    Ok(())
}

pub fn update(app: &AppHandle, view: &StatusView) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let _ = tray.set_icon(Some(icon_for(view.level)));
    let count = view.alerts.len();
    // macOS shows the number of down sites next to the dot.
    let _ = tray.set_title((count > 0).then(|| count.to_string()));
    let _ = tray.set_tooltip(Some(status_line(view)));
    match build_menu(app, view) {
        Ok(menu) => {
            let _ = tray.set_menu(Some(menu));
        }
        Err(e) => log::warn!("failed to build tray menu: {e}"),
    }
}

fn status_line(view: &StatusView) -> String {
    let checked = view
        .last_check
        .map(|t| t.with_timezone(&Local).format("%H:%M").to_string());
    match view.level {
        Level::NoKey => s().not_connected.into(),
        Level::Unknown => s().checking.into(),
        Level::Down => sites_offline(view.alerts.len()),
        Level::Warning => s().possibly_down.into(),
        Level::Error => match view.error {
            Some(crate::state::ErrorKind::InvalidKey) => s().invalid_key.into(),
            _ => s().cannot_reach.into(),
        },
        Level::Ok => match checked {
            Some(t) => format!("{} · {} {t}", s().all_up, s().checked),
            None => s().all_up.into(),
        },
    }
}

fn build_menu(app: &AppHandle, view: &StatusView) -> tauri::Result<Menu<Wry>> {
    let now = Utc::now();
    let mut menu = MenuBuilder::new(app).item(
        &MenuItemBuilder::with_id("status", status_line(view))
            .enabled(false)
            .build(app)?,
    );

    for monitor in view.alerts.iter().chain(&view.warnings) {
        let mut label = format!("● {}", monitor.name);
        if let Some(reason) = &monitor.reason {
            label.push_str(&format!(" — {reason}"));
        }
        if let Some(since) = monitor.down_since {
            label.push_str(&format!(" ({})", duration_since(since, now)));
        }
        if label.chars().count() > MAX_LABEL_CHARS {
            label = label.chars().take(MAX_LABEL_CHARS - 1).collect::<String>() + "…";
        }
        menu = menu.item(&MenuItemBuilder::with_id(format!("monitor:{}", monitor.id), label).build(app)?);
    }

    let mute_label = match view.muted_until {
        Some(until) => format!(
            "{} {}",
            s().muted_until,
            until.with_timezone(&Local).format("%H:%M")
        ),
        None => s().mute.into(),
    };
    let mut mute = SubmenuBuilder::new(app, mute_label)
        .text("mute:15", s().minutes_15)
        .text("mute:30", s().minutes_30)
        .text("mute:60", s().hour_1);
    if view.muted_until.is_some() {
        mute = mute.separator().text("unmute", s().unmute);
    }

    menu.separator()
        .text("dashboard", s().open_dashboard)
        .text("check", s().check_now)
        .item(&mute.build()?)
        .separator()
        .text("settings", s().settings)
        .text("about", "About")
        .text("quit", s().quit)
        .build()
}

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    let id = event.id().as_ref();
    match id {
        "dashboard" => ui::open_url(app, ui::DASHBOARD_URL),
        "check" => app.state::<AppState>().poll_now(),
        "settings" => ui::open_settings(app),
        "about" => ui::open_about(app),
        "unmute" => ui::unmute(app),
        "quit" => app.exit(0),
        _ => {
            if let Some(minutes) = id.strip_prefix("mute:").and_then(|m| m.parse().ok()) {
                ui::mute(app, minutes);
            } else if let Some(monitor_id) = id.strip_prefix("monitor:").and_then(|m| m.parse().ok()) {
                ui::open_url(app, &ui::monitor_url(monitor_id));
            }
        }
    }
}

fn icon_for(level: Level) -> Image<'static> {
    let rgb = match level {
        Level::Down => [0xE5, 0x48, 0x4D],
        Level::Warning => [0xF5, 0xA5, 0x24],
        Level::Ok => [0x30, 0xA4, 0x6C],
        Level::Error | Level::NoKey | Level::Unknown => [0x8B, 0x8D, 0x98],
    };
    dot_icon(rgb)
}

/// Renders an anti-aliased filled circle so no icon assets are needed.
fn dot_icon([r, g, b]: [u8; 3]) -> Image<'static> {
    let center = ICON_SIZE as f32 / 2.0;
    let radius = ICON_SIZE as f32 * 0.34;
    let mut rgba = Vec::with_capacity((ICON_SIZE * ICON_SIZE * 4) as usize);
    for y in 0..ICON_SIZE {
        for x in 0..ICON_SIZE {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let coverage = (radius + 0.5 - (dx * dx + dy * dy).sqrt()).clamp(0.0, 1.0);
            rgba.extend_from_slice(&[r, g, b, (coverage * 255.0).round() as u8]);
        }
    }
    Image::new_owned(rgba, ICON_SIZE, ICON_SIZE)
}
