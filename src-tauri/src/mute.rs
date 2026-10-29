use std::collections::BTreeSet;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Temporarily hides the widget. A mute only covers the monitors that were
/// down when it was set: any additional outage ends it immediately.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Mute {
    until: Option<DateTime<Utc>>,
    acknowledged: BTreeSet<u64>,
}

impl Mute {
    pub fn set(&mut self, duration: Duration, now: DateTime<Utc>, down_ids: impl IntoIterator<Item = u64>) {
        self.until = Some(now + duration);
        self.acknowledged = down_ids.into_iter().collect();
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn until(&self) -> Option<DateTime<Utc>> {
        self.until
    }

    /// Ends the mute when it has expired or a new monitor went down.
    /// Returns true if the mute state changed.
    pub fn refresh(&mut self, now: DateTime<Utc>, down_ids: impl IntoIterator<Item = u64>) -> bool {
        let Some(until) = self.until else {
            return false;
        };
        let has_new_outage = down_ids.into_iter().any(|id| !self.acknowledged.contains(&id));
        if until <= now || has_new_outage {
            self.clear();
            return true;
        }
        false
    }

    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        self.until.is_some_and(|until| until > now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(minute: i64) -> DateTime<Utc> {
        "2026-10-29T10:00:00Z".parse::<DateTime<Utc>>().unwrap() + Duration::minutes(minute)
    }

    #[test]
    fn expires_after_duration() {
        let mut mute = Mute::default();
        mute.set(Duration::minutes(15), at(0), [1]);
        assert!(!mute.refresh(at(14), [1]));
        assert!(mute.is_active(at(14)));
        assert!(mute.refresh(at(15), [1]));
        assert!(!mute.is_active(at(15)));
    }

    #[test]
    fn new_outage_ends_mute() {
        let mut mute = Mute::default();
        mute.set(Duration::hours(1), at(0), [1, 2]);
        assert!(!mute.refresh(at(5), [2]), "recovering monitors keep the mute");
        assert!(mute.refresh(at(6), [2, 3]));
        assert!(!mute.is_active(at(6)));
    }

    #[test]
    fn refresh_without_mute_is_noop() {
        let mut mute = Mute::default();
        assert!(!mute.refresh(at(0), [1]));
        assert_eq!(mute, Mute::default());
    }
}
