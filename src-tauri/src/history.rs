//! Picked-colour history, newest first. In memory until Phase 7 stores it in SQLite.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

/// Matches the `history` table cap in SPEC.md (Phase 7).
pub const CAPACITY: usize = 500;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: u64,
    pub hex: String,
    /// Unix time in milliseconds.
    pub picked_at: i64,
    /// Where the colour came from, e.g. "picker".
    pub source: String,
}

#[derive(Debug, Default)]
pub struct History {
    inner: Mutex<(u64, VecDeque<HistoryEntry>)>,
}

impl History {
    pub fn add(&self, hex: &str, source: &str) -> HistoryEntry {
        let picked_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let mut inner = self.inner.lock().unwrap();
        inner.0 += 1;
        let entry = HistoryEntry {
            id: inner.0,
            hex: hex.to_string(),
            picked_at,
            source: source.to_string(),
        };
        inner.1.push_front(entry.clone());
        inner.1.truncate(CAPACITY);
        entry
    }

    /// Newest first, at most `limit`.
    pub fn list(&self, limit: usize) -> Vec<HistoryEntry> {
        self.inner
            .lock()
            .unwrap()
            .1
            .iter()
            .take(limit)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_first_and_capped() {
        let h = History::default();
        for i in 0..(CAPACITY + 10) {
            h.add(&format!("#{i:06X}"), "picker");
        }
        let all = h.list(usize::MAX);
        assert_eq!(all.len(), CAPACITY);
        assert_eq!(all[0].hex, format!("#{:06X}", CAPACITY + 9));
        assert!(all.windows(2).all(|w| w[0].id > w[1].id));
        assert_eq!(h.list(3).len(), 3);
    }
}
