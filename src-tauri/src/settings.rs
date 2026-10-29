use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

use crate::i18n::Lang;

const STORE_FILE: &str = "settings.json";
const SETTINGS_KEY: &str = "settings";
const MUTE_KEY: &str = "mute";

pub const MIN_INTERVAL_MINUTES: u32 = 1;
pub const MAX_INTERVAL_MINUTES: u32 = 30;
pub const DEFAULT_WIDGET_WIDTH: f64 = 280.0;
pub const DEFAULT_WIDGET_HEIGHT: f64 = 80.0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub interval_minutes: u32,
    /// Treat LOOKS_DOWN monitors like DOWN ones (red) instead of a yellow warning.
    pub count_looks_down: bool,
    /// Keep a small green widget visible while everything is up.
    pub show_when_ok: bool,
    pub widget_width: f64,
    pub widget_height: f64,
    /// None follows the system language (English if unsupported).
    pub language: Option<Lang>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            interval_minutes: 5,
            count_looks_down: false,
            show_when_ok: false,
            widget_width: DEFAULT_WIDGET_WIDTH,
            widget_height: DEFAULT_WIDGET_HEIGHT,
            language: None,
        }
    }
}

impl Settings {
    pub fn sanitized(mut self) -> Self {
        self.interval_minutes = self.interval_minutes.clamp(MIN_INTERVAL_MINUTES, MAX_INTERVAL_MINUTES);
        self.widget_height = self.widget_height.clamp(40.0, 240.0);
        self.widget_width = self.widget_width.clamp(120.0, 800.0);
        self
    }
}

pub fn load<R: Runtime, T: for<'de> Deserialize<'de> + Default>(app: &AppHandle<R>, key: &str) -> T {
    app.store(STORE_FILE)
        .ok()
        .and_then(|store| store.get(key))
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default()
}

pub fn save<R: Runtime, T: Serialize>(app: &AppHandle<R>, key: &str, value: &T) {
    match (app.store(STORE_FILE), serde_json::to_value(value)) {
        (Ok(store), Ok(json)) => {
            store.set(key, json);
            if let Err(e) = store.save() {
                log::warn!("failed to persist {key}: {e}");
            }
        }
        (Err(e), _) => log::warn!("failed to open store: {e}"),
        (_, Err(e)) => log::warn!("failed to serialize {key}: {e}"),
    }
}

pub fn load_settings<R: Runtime>(app: &AppHandle<R>) -> Settings {
    load::<R, Settings>(app, SETTINGS_KEY).sanitized()
}

pub fn save_settings<R: Runtime>(app: &AppHandle<R>, settings: &Settings) {
    save(app, SETTINGS_KEY, settings);
}

pub fn load_mute<R: Runtime>(app: &AppHandle<R>) -> crate::mute::Mute {
    load(app, MUTE_KEY)
}

pub fn save_mute<R: Runtime>(app: &AppHandle<R>, mute: &crate::mute::Mute) {
    save(app, MUTE_KEY, mute);
}
