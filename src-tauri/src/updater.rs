use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::state::AppState;
use crate::tray;

const FIRST_CHECK_DELAY: Duration = Duration::from_secs(30);
const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Default)]
pub struct Updates {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    available: Option<Update>,
    installing: bool,
}

impl Updates {
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// What the tray menu shows about updates.
pub enum MenuState {
    None,
    Available(String),
    Installing,
}

pub fn menu_state(app: &AppHandle) -> MenuState {
    let updates = app.state::<Updates>();
    let inner = updates.lock();
    match (&inner.available, inner.installing) {
        (_, true) => MenuState::Installing,
        (Some(update), false) => MenuState::Available(update.version.clone()),
        (None, false) => MenuState::None,
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum CheckResult {
    UpToDate,
    Available { version: String },
}

/// Checks periodically in the background; debug builds never check.
pub fn spawn_checker(app: AppHandle) {
    if cfg!(debug_assertions) {
        return;
    }
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_CHECK_DELAY).await;
        loop {
            if let Err(e) = check(&app).await {
                log::warn!("update check failed: {e}");
            }
            tokio::time::sleep(CHECK_INTERVAL).await;
        }
    });
}

pub async fn check(app: &AppHandle) -> tauri_plugin_updater::Result<CheckResult> {
    let update = app.updater()?.check().await?;
    let result = match &update {
        Some(update) => CheckResult::Available { version: update.version.clone() },
        None => CheckResult::UpToDate,
    };
    app.state::<Updates>().lock().available = update;
    refresh_tray(app);
    Ok(result)
}

/// Downloads and installs the pending update, then restarts the app.
pub async fn install(app: &AppHandle) -> tauri_plugin_updater::Result<()> {
    let update = {
        let updates = app.state::<Updates>();
        let mut inner = updates.lock();
        if inner.installing {
            return Ok(());
        }
        let Some(update) = inner.available.clone() else {
            return Ok(());
        };
        inner.installing = true;
        update
    };
    refresh_tray(app);

    let result = update.download_and_install(|_, _| {}, || {}).await;
    if let Err(e) = result {
        app.state::<Updates>().lock().installing = false;
        refresh_tray(app);
        return Err(e);
    }
    app.restart();
}

pub fn spawn_install(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = install(&app).await {
            log::warn!("update installation failed: {e}");
        }
    });
}

fn refresh_tray(app: &AppHandle) {
    let view = app.state::<AppState>().lock().view(Utc::now());
    tray::update(app, &view);
}
