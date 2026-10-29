use chrono::Utc;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};
use tauri_plugin_opener::OpenerExt;

use crate::i18n::{self, s, Lang};
use crate::settings::{self, Settings};
use crate::state::AppState;
use crate::tray;

pub const WIDGET: &str = "widget";
pub const ONBOARDING: &str = "onboarding";
pub const SETTINGS: &str = "settings";
pub const DASHBOARD_URL: &str = "https://dashboard.uptimerobot.com/monitors";
pub const API_KEYS_URL: &str = "https://dashboard.uptimerobot.com/integrations";
pub const STATUS_EVENT: &str = "status-changed";
pub const LANGUAGE_EVENT: &str = "language-changed";
const SCREEN_MARGIN: f64 = 16.0;
const MENU_BAR_OFFSET: f64 = 40.0;

pub fn monitor_url(id: u64) -> String {
    format!("https://dashboard.uptimerobot.com/monitors/{id}")
}

pub fn open_url(app: &AppHandle, url: &str) {
    if let Err(e) = app.opener().open_url(url, None::<&str>) {
        log::warn!("failed to open {url}: {e}");
    }
}

/// Pushes the current status to all windows and the tray if it changed.
pub fn refresh(app: &AppHandle) {
    let state = app.state::<AppState>();
    let view = {
        let mut inner = state.lock();
        let view = inner.view(Utc::now());
        if inner.last_view.as_ref() == Some(&view) {
            return;
        }
        inner.last_view = Some(view.clone());
        view
    };

    let _ = app.emit(STATUS_EVENT, &view);
    tray::update(app, &view);

    if let Some(widget) = app.get_webview_window(WIDGET) {
        let visible = widget.is_visible().unwrap_or(false);
        if view.widget_visible && !visible {
            let _ = widget.show();
            let _ = widget.set_always_on_top(true);
        } else if !view.widget_visible && visible {
            let _ = widget.hide();
        }
    }
}

pub fn mute(app: &AppHandle, minutes: i64) {
    let state = app.state::<AppState>();
    {
        let mut inner = state.lock();
        inner.mute_for(chrono::Duration::minutes(minutes));
        settings::save_mute(app, &inner.mute);
    }
    refresh(app);
}

pub fn unmute(app: &AppHandle) {
    let state = app.state::<AppState>();
    {
        let mut inner = state.lock();
        inner.mute.clear();
        settings::save_mute(app, &inner.mute);
    }
    refresh(app);
}

/// A menu-bar-only (accessory) app cannot bring its windows to the front on
/// macOS, so it behaves like a regular app while a dialog window is open.
#[cfg(target_os = "macos")]
fn set_dialog_mode(app: &AppHandle, open: bool) {
    use tauri::ActivationPolicy;
    let policy = if open { ActivationPolicy::Regular } else { ActivationPolicy::Accessory };
    if let Err(e) = app.set_activation_policy(policy) {
        log::warn!("failed to switch activation policy: {e}");
    }
    if open {
        // Unbundled (dev) binaries have no icon of their own in the Dock.
        let _ = app.run_on_main_thread(set_dock_icon);
    }
}

#[cfg(target_os = "macos")]
fn set_dock_icon() {
    use objc2::{AllocAnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;

    const ICON: &[u8] = include_bytes!("../icons/icon.png");
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let data = NSData::with_bytes(ICON);
    if let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) {
        unsafe { NSApplication::sharedApplication(mtm).setApplicationIconImage(Some(&image)) };
    }
}

#[cfg(not(target_os = "macos"))]
fn set_dialog_mode(_app: &AppHandle, _open: bool) {}

fn open_window(app: &AppHandle, label: &str, title: &str, width: f64, height: f64) {
    set_dialog_mode(app, true);
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let result = WebviewWindowBuilder::new(app, label, WebviewUrl::App(label.into()))
        .title(title)
        .inner_size(width, height)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .center()
        .focused(true)
        .build();
    match result {
        Ok(window) => {
            let _ = window.set_focus();
            let handle = app.clone();
            let own_label = label.to_owned();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Destroyed = event {
                    let other_dialog_open = [ONBOARDING, SETTINGS]
                        .iter()
                        .any(|l| *l != own_label && handle.get_webview_window(l).is_some());
                    if !other_dialog_open {
                        set_dialog_mode(&handle, false);
                    }
                }
            });
        }
        Err(e) => log::error!("failed to open {label} window: {e}"),
    }
}

pub fn open_onboarding(app: &AppHandle) {
    open_window(app, ONBOARDING, s().setup_title, 520.0, 560.0);
}

pub fn open_settings(app: &AppHandle) {
    if app.state::<AppState>().lock().key.is_none() {
        open_onboarding(app);
    } else {
        open_window(app, SETTINGS, s().settings_title, 520.0, 720.0);
    }
}

/// Switches the UI language; open windows reload themselves on the event.
pub fn apply_language(app: &AppHandle, lang: Lang) {
    i18n::set_language(lang);
    for (label, title) in [(ONBOARDING, s().setup_title), (SETTINGS, s().settings_title)] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.set_title(title);
        }
    }
    let _ = app.emit(LANGUAGE_EVENT, lang.code());
}

pub fn close_window(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.close();
    }
}

pub fn apply_widget_size(app: &AppHandle, settings: &Settings) {
    if let Some(widget) = app.get_webview_window(WIDGET) {
        let _ = widget.set_size(LogicalSize::new(settings.widget_width, settings.widget_height));
    }
}

/// Restores the saved widget position, or places it in the top-right corner
/// of the primary screen when there is none or its screen is gone.
pub fn place_widget(app: &AppHandle, widget: &WebviewWindow) {
    let saved: Option<(i32, i32)> = settings::load(app, "widgetPosition");
    let monitors = widget.available_monitors().unwrap_or_default();
    if let Some((x, y)) = saved {
        let on_screen = monitors.iter().any(|m| {
            let (pos, size) = (m.position(), m.size());
            x >= pos.x && y >= pos.y && x < pos.x + size.width as i32 - 20 && y < pos.y + size.height as i32 - 20
        });
        if on_screen {
            let _ = widget.set_position(PhysicalPosition::new(x, y));
            return;
        }
    }

    let Some(monitor) = widget.primary_monitor().ok().flatten().or_else(|| monitors.into_iter().next()) else {
        return;
    };
    let scale = monitor.scale_factor();
    let width = widget.outer_size().map(|s| s.width as f64).unwrap_or(280.0 * scale);
    let x = monitor.position().x as f64 + monitor.size().width as f64 - width - SCREEN_MARGIN * scale;
    let y = monitor.position().y as f64 + MENU_BAR_OFFSET * scale;
    let _ = widget.set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32));
}
