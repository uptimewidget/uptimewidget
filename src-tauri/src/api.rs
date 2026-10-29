use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

const API_BASE: &str = "https://api.uptimerobot.com/v3";
const PAGE_LIMIT: u32 = 200;
const DEFAULT_RETRY_AFTER_SECS: u64 = 60;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ApiError {
    #[error("invalid API key")]
    InvalidKey,
    #[error("rate limited, retry after {0}s")]
    RateLimited(u64),
    #[error("network error: {0}")]
    Network(String),
    #[error("unexpected response: {0}")]
    Unexpected(String),
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MonitorStatus {
    Down,
    LooksDown,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    pub id: u64,
    pub name: String,
    pub url: Option<String>,
    pub status: MonitorStatus,
    pub reason: Option<String>,
    pub down_since: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub email: Option<String>,
    pub monitors_count: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MonitorPage {
    #[serde(default)]
    next_link: Option<String>,
    data: Vec<RawMonitor>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawMonitor {
    id: u64,
    friendly_name: String,
    url: Option<String>,
    status: String,
    current_state_duration: Option<i64>,
    last_incident: Option<RawIncident>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawIncident {
    status: Option<String>,
    reason: Option<String>,
    started_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawUser {
    email: Option<String>,
    monitors_count: u32,
}

impl RawMonitor {
    fn into_monitor(self, now: DateTime<Utc>) -> Option<Monitor> {
        let status = match self.status.to_ascii_uppercase().as_str() {
            "DOWN" => MonitorStatus::Down,
            "LOOKS_DOWN" => MonitorStatus::LooksDown,
            _ => return None,
        };
        // Only trust incident details while the incident is still ongoing.
        let incident = self
            .last_incident
            .filter(|i| i.status.as_deref().is_none_or(|s| s.eq_ignore_ascii_case("ongoing")));
        let down_since = incident
            .as_ref()
            .and_then(|i| i.started_at)
            .or_else(|| {
                self.current_state_duration
                    .map(|secs| now - chrono::Duration::seconds(secs))
            });
        Some(Monitor {
            id: self.id,
            name: self.friendly_name,
            url: self.url.filter(|u| !u.is_empty()),
            status,
            reason: incident.and_then(|i| i.reason).filter(|r| !r.is_empty()),
            down_since,
        })
    }
}

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent(concat!("UptimeWidget/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("failed to build HTTP client");
        Self { http }
    }

    /// Returns all monitors that are currently DOWN or LOOKS_DOWN.
    pub async fn fetch_problem_monitors(&self, key: &str) -> Result<Vec<Monitor>, ApiError> {
        let mut url = format!("{API_BASE}/monitors?status=DOWN,LOOKS_DOWN&limit={PAGE_LIMIT}");
        let mut result = Vec::new();
        loop {
            let page: MonitorPage = self.get_json(&url, key).await?;
            let now = Utc::now();
            result.extend(page.data.into_iter().filter_map(|m| m.into_monitor(now)));
            match page.next_link {
                // Never send the key to a host other than the API.
                Some(next) if next.starts_with(API_BASE) => url = next,
                _ => break,
            }
        }
        Ok(result)
    }

    pub async fn account_info(&self, key: &str) -> Result<AccountInfo, ApiError> {
        let user: RawUser = self.get_json(&format!("{API_BASE}/user/me"), key).await?;
        Ok(AccountInfo {
            email: user.email,
            monitors_count: user.monitors_count,
        })
    }

    async fn get_json<T: for<'de> Deserialize<'de>>(&self, url: &str, key: &str) -> Result<T, ApiError> {
        let response = self
            .http
            .get(url)
            .bearer_auth(key)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        match response.status() {
            s if s.is_success() => response
                .json::<T>()
                .await
                .map_err(|e| ApiError::Unexpected(e.to_string())),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(ApiError::InvalidKey),
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.trim().parse().ok())
                    .unwrap_or(DEFAULT_RETRY_AFTER_SECS);
                Err(ApiError::RateLimited(retry_after))
            }
            s => Err(ApiError::Unexpected(format!("HTTP {s}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{
      "data": [
        {
          "id": 804285578, "friendlyName": "cms.kleeblatt.app", "status": "DOWN",
          "url": "cms.kleeblatt.app", "currentStateDuration": 1185178, "type": "HTTP",
          "lastIncident": { "id": "1", "status": "Ongoing", "cause": 502,
            "reason": "502 Bad Gateway", "startedAt": "2026-10-15T16:07:21.135Z", "duration": null }
        },
        {
          "id": 2, "friendlyName": "flaky.example", "status": "LOOKS_DOWN",
          "url": "", "currentStateDuration": 120,
          "lastIncident": { "id": "2", "status": "Resolved", "reason": "old", "startedAt": "2025-01-01T00:00:00Z" }
        },
        { "id": 3, "friendlyName": "up.example", "status": "UP", "url": "up.example",
          "currentStateDuration": 10, "lastIncident": null }
      ]
    }"#;

    #[test]
    fn parses_monitor_page() {
        let page: MonitorPage = serde_json::from_str(FIXTURE).unwrap();
        assert!(page.next_link.is_none());
        let now: DateTime<Utc> = "2026-10-29T10:00:00Z".parse().unwrap();
        let monitors: Vec<Monitor> = page.data.into_iter().filter_map(|m| m.into_monitor(now)).collect();
        assert_eq!(monitors.len(), 2);

        let down = &monitors[0];
        assert_eq!(down.id, 804285578);
        assert_eq!(down.status, MonitorStatus::Down);
        assert_eq!(down.reason.as_deref(), Some("502 Bad Gateway"));
        assert_eq!(down.down_since, Some("2026-10-15T16:07:21.135Z".parse().unwrap()));

        let flaky = &monitors[1];
        assert_eq!(flaky.status, MonitorStatus::LooksDown);
        assert_eq!(flaky.url, None);
        assert_eq!(flaky.reason, None, "resolved incident must not be used");
        assert_eq!(flaky.down_since, Some(now - chrono::Duration::seconds(120)));
    }
}
