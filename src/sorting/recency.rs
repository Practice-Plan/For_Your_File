//! Recency-based sorting algorithm
//!
//! Sorts entries by last opened timestamp with configurable time windows
//! and relative time indicators.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::TimeWindow;
use crate::models::Entry;

/// Recency scorer for sorting by last opened time
#[derive(Debug, Clone)]
pub struct RecencyScorer {
    /// Reference timestamp (usually now)
    reference_time: i64,
}

impl Default for RecencyScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl RecencyScorer {
    /// Create a new recency scorer using current time as reference
    pub fn new() -> Self {
        Self {
            reference_time: Utc::now().timestamp(),
        }
    }

    /// Create a recency scorer with a specific reference time
    pub fn with_reference_time(reference_time: i64) -> Self {
        Self { reference_time }
    }

    /// Get the reference time
    pub fn reference_time(&self) -> i64 {
        self.reference_time
    }

    /// Update the reference time to now
    pub fn update_reference_time(&mut self) {
        self.reference_time = Utc::now().timestamp();
    }

    /// Calculate recency score (higher is more recent)
    ///
    /// Uses exponential decay from now:
    /// score = e^(-hours_ago / 24) where 24 is the decay constant
    pub fn calculate_score(&self, entry: &Entry) -> f64 {
        match entry.last_opened {
            Some(timestamp) => {
                let hours_ago = (self.reference_time - timestamp) as f64 / 3600.0;
                if hours_ago <= 0.0 {
                    1.0 // Just opened now
                } else {
                    // Exponential decay: e^(-hours/24)
                    // Gives 1.0 at 0 hours, ~0.61 at 12 hours, ~0.37 at 24 hours
                    (-hours_ago / 24.0).exp()
                }
            }
            None => 0.0, // Never opened
        }
    }

    /// Get the time window for an entry
    pub fn get_time_window(&self, entry: &Entry) -> TimeWindow {
        match entry.last_opened {
            Some(timestamp) => TimeWindow::from_timestamp(timestamp, self.reference_time),
            None => TimeWindow::Older,
        }
    }

    /// Get relative time string for display
    ///
    /// Returns human-readable strings like "Just now", "5 minutes ago", etc.
    pub fn get_relative_time(&self, entry: &Entry) -> String {
        match entry.last_opened {
            Some(timestamp) => self.format_relative_time(timestamp),
            None => "Never opened".to_string(),
        }
    }

    /// Format a timestamp as relative time
    fn format_relative_time(&self, timestamp: i64) -> String {
        let now = self.reference_time;
        let diff_seconds = now - timestamp;

        if diff_seconds < 0 {
            return "In the future".to_string();
        }

        let minutes = diff_seconds / 60;
        let hours = minutes / 60;
        let days = hours / 24;
        let weeks = days / 7;
        let months = days / 30;
        let years = days / 365;

        if minutes < 1 {
            "Just now".to_string()
        } else if minutes < 60 {
            format!("{} minute{} ago", minutes, if minutes == 1 { "" } else { "s" })
        } else if hours < 24 {
            format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
        } else if days < 7 {
            format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
        } else if weeks < 4 {
            format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
        } else if months < 12 {
            format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
        } else {
            format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
        }
    }

    /// Sort entries by recency (most recent first)
    pub fn sort_by_recency(&self, entries: Vec<Entry>) -> Vec<Entry> {
        let mut sorted = entries;
        sorted.sort_by(|a, b| {
            // Sort by last_opened descending (most recent first)
            // Never opened entries go to the end
            match (a.last_opened, b.last_opened) {
                (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
        sorted
    }

    /// Group entries by time window
    pub fn group_by_time_window(&self, entries: Vec<Entry>) -> Vec<(TimeWindow, Vec<Entry>)> {
        let mut groups: std::collections::HashMap<TimeWindow, Vec<Entry>> =
            std::collections::HashMap::new();

        for entry in entries {
            let window = self.get_time_window(&entry);
            groups.entry(window).or_default().push(entry);
        }

        // Convert to sorted vector
        let order = [
            TimeWindow::Hour,
            TimeWindow::Day,
            TimeWindow::Week,
            TimeWindow::Month,
            TimeWindow::Older,
        ];

        order
            .iter()
            .filter_map(|window| {
                let entries = groups.remove(window)?;
                if !entries.is_empty() {
                    Some((*window, entries))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Entry grouped by time window for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupedEntries {
    /// Time window label
    pub window: String,
    /// Entries in this group
    pub entries: Vec<Entry>,
    /// Relative time indicators for each entry
    pub relative_times: Vec<String>,
}

impl GroupedEntries {
    /// Create grouped entries from a time window and entries
    pub fn new(scorer: &RecencyScorer, window: TimeWindow, entries: Vec<Entry>) -> Self {
        let relative_times: Vec<String> =
            entries.iter().map(|e| scorer.get_relative_time(e)).collect();

        Self {
            window: window.label().to_string(),
            entries,
            relative_times,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(id: i64, last_opened: Option<i64>) -> Entry {
        Entry {
            id: Some(id),
            lnk_path: format!("test_{}.lnk", id),
            target_path: format!("target_{}.exe", id),
            parameters: None,
            working_dir: None,
            tags: None,
            notes: None,
            frequency: 1,
            last_opened,
            created_at: 0,
            updated_at: 0,
            group_id: None,
            expires_at: None,
            target_type: crate::models::LnkTarget::File(format!("target_{}.exe", id)),
            description: None,
            icon_location: None,
            icon_index: None,
        }
    }

    #[test]
    fn test_recency_score_just_now() {
        let now = Utc::now().timestamp();
        let scorer = RecencyScorer::with_reference_time(now);
        let entry = create_test_entry(1, Some(now));
        let score = scorer.calculate_score(&entry);
        assert!((score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_recency_score_never_opened() {
        let scorer = RecencyScorer::new();
        let entry = create_test_entry(1, None);
        let score = scorer.calculate_score(&entry);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_recency_score_one_hour_ago() {
        let now = Utc::now().timestamp();
        let scorer = RecencyScorer::with_reference_time(now);
        let entry = create_test_entry(1, Some(now - 3600));
        let score = scorer.calculate_score(&entry);
        // After 1 hour: e^(-1/24) ≈ 0.96
        assert!(score > 0.95 && score < 0.97);
    }

    #[test]
    fn test_recency_score_one_day_ago() {
        let now = Utc::now().timestamp();
        let scorer = RecencyScorer::with_reference_time(now);
        let entry = create_test_entry(1, Some(now - 86400));
        let score = scorer.calculate_score(&entry);
        // After 24 hours: e^(-24/24) = e^(-1) ≈ 0.37
        assert!((score - 0.37).abs() < 0.02);
    }

    #[test]
    fn test_relative_time_just_now() {
        let now = Utc::now().timestamp();
        let scorer = RecencyScorer::with_reference_time(now);
        let entry = create_test_entry(1, Some(now - 30));
        let relative = scorer.get_relative_time(&entry);
        assert_eq!(relative, "Just now");
    }

    #[test]
    fn test_relative_time_minutes() {
        let now = Utc::now().timestamp();
        let scorer = RecencyScorer::with_reference_time(now);
        let entry = create_test_entry(1, Some(now - 300));
        let relative = scorer.get_relative_time(&entry);
        assert_eq!(relative, "5 minutes ago");
    }

    #[test]
    fn test_relative_time_hours() {
        let now = Utc::now().timestamp();
        let scorer = RecencyScorer::with_reference_time(now);
        let entry = create_test_entry(1, Some(now - 7200));
        let relative = scorer.get_relative_time(&entry);
        assert_eq!(relative, "2 hours ago");
    }

    #[test]
    fn test_relative_time_days() {
        let now = Utc::now().timestamp();
        let scorer = RecencyScorer::with_reference_time(now);
        let entry = create_test_entry(1, Some(now - 172800));
        let relative = scorer.get_relative_time(&entry);
        assert_eq!(relative, "2 days ago");
    }

    #[test]
    fn test_sort_by_recency() {
        let now = Utc::now().timestamp();
        let scorer = RecencyScorer::with_reference_time(now);

        let entries = vec![
            create_test_entry(1, Some(now - 86400)), // 1 day ago
            create_test_entry(2, Some(now - 3600)),  // 1 hour ago
            create_test_entry(3, Some(now)),         // Just now
            create_test_entry(4, None),              // Never opened
        ];

        let sorted = scorer.sort_by_recency(entries);

        assert_eq!(sorted[0].id, Some(3)); // Most recent
        assert_eq!(sorted[1].id, Some(2));
        assert_eq!(sorted[2].id, Some(1));
        assert_eq!(sorted[3].id, Some(4)); // Never opened at the end
    }

    #[test]
    fn test_time_window_classification() {
        let now = Utc::now().timestamp();
        let scorer = RecencyScorer::with_reference_time(now);

        let entry_hour = create_test_entry(1, Some(now - 1800)); // 30 min ago
        let entry_day = create_test_entry(2, Some(now - 3600 * 12)); // 12 hours ago
        let entry_week = create_test_entry(3, Some(now - 86400 * 3)); // 3 days ago
        let entry_month = create_test_entry(4, Some(now - 86400 * 15)); // 15 days ago
        let entry_older = create_test_entry(5, Some(now - 86400 * 60)); // 60 days ago

        assert_eq!(scorer.get_time_window(&entry_hour), TimeWindow::Hour);
        assert_eq!(scorer.get_time_window(&entry_day), TimeWindow::Day);
        assert_eq!(scorer.get_time_window(&entry_week), TimeWindow::Week);
        assert_eq!(scorer.get_time_window(&entry_month), TimeWindow::Month);
        assert_eq!(scorer.get_time_window(&entry_older), TimeWindow::Older);
    }

    #[test]
    fn test_group_by_time_window() {
        let now = Utc::now().timestamp();
        let scorer = RecencyScorer::with_reference_time(now);

        let entries = vec![
            create_test_entry(1, Some(now - 1800)),      // Hour
            create_test_entry(2, Some(now - 3600 * 12)), // Day
            create_test_entry(3, Some(now - 3600)),      // Hour
        ];

        let groups = scorer.group_by_time_window(entries);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, TimeWindow::Hour);
        assert_eq!(groups[0].1.len(), 2);
        assert_eq!(groups[1].0, TimeWindow::Day);
        assert_eq!(groups[1].1.len(), 1);
    }
}