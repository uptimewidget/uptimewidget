use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tauri::{AppHandle, Manager, PhysicalPosition};
use tokio::sync::Notify;
use tokio::time::Instant;

use crate::api::{self, ApiError, Monitor, MonitorStatus};
use crate::mute::Mute;
use crate::settings::{self, Settings};
use crate::ui;

const TICK: Duration = Duration::from_secs(15);
const NETWORK_RETRY: Duration = Duration::from_secs(60);
/// Network errors are only surfaced once data is older than this (or 3 intervals).
const MIN_STALE_AFTER: chrono::Duration = chrono::Duration::minutes(15);

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ErrorKind {
    InvalidKey,
    Network,
    RateLimited,
    Unexpected,
}

impl From<&ApiError> for ErrorKind {
    fn from(e: &ApiError) -> Self {
        match e {
            ApiError::InvalidKey => Self::InvalidKey,
            ApiError::RateLimited(_) => Self::RateLimited,
            ApiError::Network(_) => Self::Network,
            ApiError::Unexpected(_) => Self::Unexpected,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Level {
    NoKey,
    Unknown,
    Ok,
    Warning,
    Down,
    Error,
}

/// Everything the UI (widget, tray, settings) needs to render.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatusView {
    pub level: Level,
    pub alerts: Vec<Monitor>,
    pub warnings: Vec<Monitor>,
    pub error: Option<ErrorKind>,
    pub last_check: Option<DateTime<Utc>>,
    pub muted_until: Option<DateTime<Utc>>,
    pub checking: bool,
    pub widget_visible: bool,
}

pub struct Inner {
    pub key: Option<String>,
    pub settings: Settings,
    pub mute: Mute,
    pub monitors: Vec<Monitor>,
    pub error: Option<ErrorKind>,
    pub last_check: Option<DateTime<Utc>>,
    pub checking: bool,
    /// None means a poll is due right away.
    pub next_poll_at: Option<Instant>,
    pub last_view: Option<StatusView>,
    pub widget_position: Option<PhysicalPosition<i32>>,
    pub widget_position_dirty: bool,
}

impl Inner {
    fn split_monitors(&self) -> (Vec<Monitor>, Vec<Monitor>) {
        self.monitors.iter().cloned().partition(|m| {
            m.status == MonitorStatus::Down || self.settings.count_looks_down
        })
    }

    fn alert_ids(&self) -> Vec<u64> {
        self.split_monitors().0.iter().map(|m| m.id).collect()
    }

    pub fn mute_for(&mut self, duration: chrono::Duration) {
        let ids: Vec<u64> = self.monitors.iter().map(|m| m.id).collect();
        self.mute.set(duration, Utc::now(), ids);
    }

    pub fn view(&self, now: DateTime<Utc>) -> StatusView {
        let (alerts, warnings) = self.split_monitors();
        let stale_after = MIN_STALE_AFTER.max(chrono::Duration::minutes(3 * i64::from(self.settings.interval_minutes)));
        let stale = self.last_check.is_none_or(|t| now - t > stale_after);

        let level = if self.key.is_none() {
            Level::NoKey
        } else if self.error == Some(ErrorKind::InvalidKey) {
            Level::Error
        } else if !alerts.is_empty() {
            Level::Down
        } else if !warnings.is_empty() {
            Level::Warning
        } else if self.error.is_some() && stale {
            Level::Error
        } else if self.last_check.is_none() {
            Level::Unknown
        } else {
            Level::Ok
        };

        let muted = self.mute.is_active(now);
        let widget_visible = !muted
            && match level {
                Level::Down | Level::Warning | Level::Error => true,
                Level::Ok => self.settings.show_when_ok,
                Level::NoKey | Level::Unknown => false,
            };

        StatusView {
            level,
            alerts,
            warnings,
            error: self.error,
            last_check: self.last_check,
            muted_until: self.mute.until().filter(|_| muted),
            checking: self.checking,
            widget_visible,
        }
    }
}

pub struct AppState {
    inner: Mutex<Inner>,
    wake: Notify,
    client: api::Client,
}

impl AppState {
    pub fn new(key: Option<String>, settings: Settings, mute: Mute) -> Self {
        Self {
            inner: Mutex::new(Inner {
                key,
                settings,
                mute,
                monitors: Vec::new(),
                error: None,
                last_check: None,
                checking: false,
                next_poll_at: None,
                last_view: None,
                widget_position: None,
                widget_position_dirty: false,
            }),
            wake: Notify::new(),
            client: api::Client::new(),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn client(&self) -> &api::Client {
        &self.client
    }

    /// Schedules an immediate poll and wakes the poller.
    pub fn poll_now(&self) {
        self.lock().next_poll_at = None;
        self.wake.notify_one();
    }
}

pub fn spawn_poller(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let state = app.state::<AppState>();
            let due_key = {
                let mut inner = state.lock();
                let due = inner.next_poll_at.is_none_or(|at| at <= Instant::now());
                match inner.key.clone() {
                    Some(key) if due => {
                        inner.checking = true;
                        Some(key)
                    }
                    _ => None,
                }
            };

            if let Some(key) = due_key {
                ui::refresh(&app);
                poll(&app, &state, key).await;
            }

            {
                let mut inner = state.lock();
                let ids = inner.alert_ids();
                if inner.mute.refresh(Utc::now(), ids) {
                    settings::save_mute(&app, &inner.mute);
                }
                if inner.widget_position_dirty {
                    inner.widget_position_dirty = false;
                    if let Some(pos) = inner.widget_position {
                        settings::save(&app, "widgetPosition", &(pos.x, pos.y));
                    }
                }
            }
            ui::refresh(&app);

            tokio::select! {
                _ = tokio::time::sleep(TICK) => {}
                _ = state.wake.notified() => {}
            }
        }
    });
}

async fn poll(app: &AppHandle, state: &AppState, key: String) {
    let result = state.client().fetch_problem_monitors(&key).await;

    let mut inner = state.lock();
    inner.checking = false;
    if inner.key.as_deref() != Some(key.as_str()) {
        // The key changed while this request was in flight; discard the result.
        return;
    }
    let interval = Duration::from_secs(60 * u64::from(inner.settings.interval_minutes));
    let next_in = match result {
        Ok(monitors) => {
            inner.monitors = monitors;
            inner.error = None;
            inner.last_check = Some(Utc::now());
            interval
        }
        Err(e) => {
            log::warn!("poll failed: {e}");
            inner.error = Some(ErrorKind::from(&e));
            match e {
                ApiError::InvalidKey => {
                    inner.monitors.clear();
                    drop(inner);
                    ui::open_onboarding(app);
                    state.lock().next_poll_at = Some(Instant::now() + interval);
                    return;
                }
                ApiError::RateLimited(secs) => Duration::from_secs(secs.max(1)),
                ApiError::Network(_) | ApiError::Unexpected(_) => NETWORK_RETRY.min(interval),
            }
        }
    };
    inner.next_poll_at = Some(Instant::now() + next_in);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor(id: u64, status: MonitorStatus) -> Monitor {
        Monitor {
            id,
            name: format!("m{id}"),
            url: None,
            status,
            reason: None,
            down_since: None,
        }
    }

    fn inner() -> Inner {
        let state = AppState::new(Some("key".into()), Settings::default(), Mute::default());
        state.inner.into_inner().unwrap()
    }

    #[test]
    fn levels() {
        let now = Utc::now();
        let mut s = inner();
        assert_eq!(s.view(now).level, Level::Unknown);

        s.last_check = Some(now);
        assert_eq!(s.view(now).level, Level::Ok);
        assert!(!s.view(now).widget_visible);

        s.monitors = vec![monitor(1, MonitorStatus::LooksDown)];
        assert_eq!(s.view(now).level, Level::Warning);
        s.settings.count_looks_down = true;
        assert_eq!(s.view(now).level, Level::Down);

        s.monitors.push(monitor(2, MonitorStatus::Down));
        s.settings.count_looks_down = false;
        let view = s.view(now);
        assert_eq!(view.level, Level::Down);
        assert_eq!((view.alerts.len(), view.warnings.len()), (1, 1));
        assert!(view.widget_visible);
    }

    #[test]
    fn network_errors_surface_only_when_stale() {
        let now = Utc::now();
        let mut s = inner();
        s.last_check = Some(now - chrono::Duration::minutes(5));
        s.error = Some(ErrorKind::Network);
        assert_eq!(s.view(now).level, Level::Ok);

        s.last_check = Some(now - chrono::Duration::minutes(20));
        assert_eq!(s.view(now).level, Level::Error);
    }

    #[test]
    fn mute_hides_widget() {
        let now = Utc::now();
        let mut s = inner();
        s.last_check = Some(now);
        s.monitors = vec![monitor(1, MonitorStatus::Down)];
        s.mute_for(chrono::Duration::minutes(10));
        let view = s.view(now);
        assert!(!view.widget_visible);
        assert!(view.muted_until.is_some());
    }
}
