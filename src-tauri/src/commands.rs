use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, State};

use crate::api::{AccountInfo, ApiError};
use crate::i18n::{self, Lang};
use crate::secrets;
use crate::settings::{self, Settings};
use crate::state::{AppState, ErrorKind, StatusView};
use crate::ui;

/// Longest mute the UI offers (5 hours).
const MAX_MUTE_MINUTES: i64 = 300;

fn error_code(e: &ApiError) -> String {
    serde_json::to_value(ErrorKind::from(e))
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unexpected".into())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyInfo {
    has_key: bool,
    masked_key: Option<String>,
}

#[tauri::command]
pub fn get_status(state: State<AppState>) -> StatusView {
    state.lock().view(Utc::now())
}

#[tauri::command]
pub fn check_now(state: State<AppState>) {
    state.poll_now();
}

#[tauri::command]
pub async fn validate_key(state: State<'_, AppState>, key: String) -> Result<AccountInfo, String> {
    let client = state.client().clone();
    client.account_info(key.trim()).await.map_err(|e| error_code(&e))
}

#[tauri::command]
pub async fn save_key(app: AppHandle, state: State<'_, AppState>, key: String) -> Result<AccountInfo, String> {
    let key = key.trim().to_owned();
    let client = state.client().clone();
    let info = client.account_info(&key).await.map_err(|e| error_code(&e))?;
    secrets::store_key(&key)?;
    {
        let mut inner = state.lock();
        inner.key = Some(key);
        inner.monitors.clear();
        inner.error = None;
        inner.last_check = None;
        inner.mute.clear();
        settings::save_mute(&app, &inner.mute);
    }
    enable_autostart_on_first_setup(&app);
    state.poll_now();
    Ok(info)
}

#[cfg(not(debug_assertions))]
fn enable_autostart_on_first_setup(app: &AppHandle) {
    use tauri_plugin_autostart::ManagerExt;
    const FLAG: &str = "autostartInitialized";
    if !settings::load::<_, bool>(app, FLAG) {
        if let Err(e) = app.autolaunch().enable() {
            log::warn!("failed to enable autostart: {e}");
        }
        settings::save(app, FLAG, &true);
    }
}

#[cfg(debug_assertions)]
fn enable_autostart_on_first_setup(_app: &AppHandle) {
    // Dev builds must not register themselves as login item.
}

#[tauri::command]
pub fn remove_key(app: AppHandle, state: State<AppState>) {
    secrets::delete_key();
    {
        let mut inner = state.lock();
        inner.key = None;
        inner.monitors.clear();
        inner.error = None;
        inner.last_check = None;
    }
    ui::refresh(&app);
    ui::close_window(&app, ui::SETTINGS);
    ui::open_onboarding(&app);
}

#[tauri::command]
pub fn get_key_info(state: State<AppState>) -> KeyInfo {
    let inner = state.lock();
    let masked_key = inner.key.as_ref().map(|k| {
        let tail: String = k.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
        format!("••••••••{tail}")
    });
    KeyInfo {
        has_key: inner.key.is_some(),
        masked_key,
    }
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Settings {
    state.lock().settings.clone()
}

#[tauri::command]
pub fn save_settings(app: AppHandle, state: State<AppState>, settings: Settings) -> Settings {
    let settings = settings.sanitized();
    let (size_changed, interval_changed, language_changed) = {
        let mut inner = state.lock();
        let old = std::mem::replace(&mut inner.settings, settings.clone());
        let language_changed = Lang::resolve(old.language) != Lang::resolve(settings.language);
        if language_changed {
            // Forces the tray menu to be rebuilt in the new language.
            inner.last_view = None;
        }
        (
            old.widget_width != settings.widget_width || old.widget_height != settings.widget_height,
            old.interval_minutes != settings.interval_minutes,
            language_changed,
        )
    };
    settings::save_settings(&app, &settings);
    if size_changed {
        ui::apply_widget_size(&app, &settings);
    }
    if interval_changed {
        state.poll_now();
    }
    if language_changed {
        ui::apply_language(&app, Lang::resolve(settings.language));
    }
    ui::refresh(&app);
    settings
}

#[tauri::command]
pub fn get_language() -> &'static str {
    i18n::language().code()
}

/// Called by the widget after the user resized it by dragging its edge.
#[tauri::command]
pub fn set_widget_size(app: AppHandle, state: State<AppState>, width: f64, height: f64) {
    let settings = {
        let mut inner = state.lock();
        inner.settings.widget_width = width;
        inner.settings.widget_height = height;
        inner.settings = inner.settings.clone().sanitized();
        inner.settings.clone()
    };
    settings::save_settings(&app, &settings);
}

#[tauri::command]
pub fn mute(app: AppHandle, minutes: i64) {
    ui::mute(&app, minutes.clamp(1, MAX_MUTE_MINUTES));
}

#[tauri::command]
pub fn unmute(app: AppHandle) {
    ui::unmute(&app);
}

#[tauri::command]
pub fn open_monitor(app: AppHandle, id: u64) {
    ui::open_url(&app, &ui::monitor_url(id));
}

#[tauri::command]
pub fn open_dashboard(app: AppHandle) {
    ui::open_url(&app, ui::DASHBOARD_URL);
}

#[tauri::command]
pub fn open_api_keys_page(app: AppHandle) {
    ui::open_url(&app, ui::API_KEYS_URL);
}

#[tauri::command]
pub fn open_settings(app: AppHandle) {
    ui::open_settings(&app);
}
