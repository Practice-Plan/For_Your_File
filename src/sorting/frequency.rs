//! Frequency-based sorting algorithm
//!
//! Calculates frequency score with time decay, where recent opens
//! count more than old ones.

use chrono::Utc;

use crate::models::Entry;

/// Frequency scorer with time decay support
#[derive(Debug, Clone)]
pub struct FrequencyScorer {
    /// Half-life for decay in days
    half_life: f32,
    /// Maximum frequency to consider (for normalization)
    max_frequency: i32,
}

impl Default for FrequencyScorer {
    fn default() -> Self {
        Self::new(7.0)
    }
}

impl FrequencyScorer {
    /// Create a new frequency scorer with the given half-life
    ///
    /// # Arguments
    /// * `half_life` - Number of days for frequency to decay to half its value
    pub fn new(half_life: f32) -> Self {
        Self {
            half_life,
            max_frequency: 0,
        }
    }

    /// Set the half-life for decay
    pub fn set_half_life(&mut self, half_life: f32) {
        self.half_life = half_life;
    }

    /// Get the current half-life
    pub fn half_life(&self) -> f32 {
        self.half_life
    }

    /// Calculate decay factor based on days since last open
    ///
    /// Uses exponential decay: decay = 0.5^(days / half_life)
    fn calculate_decay(&self, days_since_open: f64) -> f64 {
        if days_since_open <= 0.0 {
            return 1.0;
        }
        0.5_f64.powf(days_since_open / self.half_life as f64)
    }

    /// Calculate frequency score for a single entry
    ///
    /// Score formula: score = frequency * decay(days_since_last_open)
    /// This gives higher weight to entries that are used frequently AND recently
    pub fn calculate_score(&self, entry: &Entry) -> f64 {
        let frequency = entry.frequency as f64;

        // If never opened, return 0
        if frequency == 0.0 {
            return 0.0;
        }

        // Calculate time decay based on last opened time
        let decay = match entry.last_opened {
            Some(timestamp) => {
                let now = Utc::now().timestamp();
                let days_since_opened = (now - timestamp) as f64 / 86400.0;
                self.calculate_decay(days_since_opened)
            }
            None => 0.0, // Never opened
        };

        frequency * decay
    }

    /// Calculate frequency score with explicit frequency and days since open
    ///
    /// Formula: score = frequency / (days_since_opened + 1)
    /// This is an alternative decay formula that penalizes old entries more gently
    pub fn calculate_score_simple(frequency: i32, days_since_opened: f64) -> f64 {
        if frequency == 0 {
            return 0.0;
        }
        frequency as f64 / (days_since_opened + 1.0)
    }

    /// Calculate cumulative frequency score over multiple opens
    ///
    /// This considers each open individually with its own decay
    /// Score = Σ(frequency_contribution / (days_since_open + 1))
    pub fn calculate_cumulative_score(&self, opens: &[(i64, i32)]) -> f64 {
        let now = Utc::now().timestamp();
        let mut total_score = 0.0;

        for (timestamp, count) in opens {
            let days_since = (now - timestamp) as f64 / 86400.0;
            let contribution = (*count as f64) / (days_since + 1.0);
            total_score += contribution;
        }

        total_score
    }

    /// Calculate scores for a batch of entries and normalize them
    ///
    /// # Arguments
    /// * `entries` - Slice of entries to score
    ///
    /// # Returns
    /// Vector of normalized scores (0.0 - 1.0) in the same order as entries
    pub fn calculate_normalized_scores(&self, entries: &[Entry]) -> Vec<f64> {
        // Calculate raw scores
        let raw_scores: Vec<f64> = entries.iter().map(|e| self.calculate_score(e)).collect();

        // Find max for normalization
        let max_score = raw_scores.iter().fold(0.0_f64, |a, b| a.max(*b));

        // Normalize to 0.0 - 1.0
        if max_score > 0.0 {
            raw_scores.iter().map(|s| s / max_score).collect()
        } else {
            raw_scores
        }
    }

    /// Sort entries by frequency score (descending)
    pub fn sort_by_frequency(&self, entries: Vec<Entry>) -> Vec<Entry> {
        let mut scored: Vec<(Entry, f64)> = entries
            .into_iter()
            .map(|e| {
                let score = self.calculate_score(&e);
                (e, score)
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Extract entries
        scored.into_iter().map(|(e, _)| e).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(id: i64, frequency: i32, last_opened: Option<i64>) -> Entry {
        Entry {
            id: Some(id),
            lnk_path: format!("test_{}.lnk", id),
            target_path: format!("target_{}.exe", id),
            parameters: None,
            working_dir: None,
            tags: None,
            notes: None,
            frequency,
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
    fn test_frequency_scorer_default() {
        let scorer = FrequencyScorer::default();
        assert_eq!(scorer.half_life(), 7.0);
    }

    #[test]
    fn test_calculate_score_never_opened() {
        let scorer = FrequencyScorer::new(7.0);
        let entry = create_test_entry(1, 10, None);
        let score = scorer.calculate_score(&entry);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_calculate_score_recent() {
        let scorer = FrequencyScorer::new(7.0);
        let now = Utc::now().timestamp();
        let entry = create_test_entry(1, 10, Some(now));
        let score = scorer.calculate_score(&entry);
        assert!(score > 9.0); // Should be close to 10 with minimal decay
    }

    #[test]
    fn test_calculate_score_old() {
        let scorer = FrequencyScorer::new(7.0);
        let now = Utc::now().timestamp();
        let thirty_days_ago = now - 86400 * 30;
        let entry = create_test_entry(1, 10, Some(thirty_days_ago));
        let score = scorer.calculate_score(&entry);

        // After 30 days with 7-day half-life: decay = 0.5^(30/7) ≈ 0.05
        // Score should be approximately 10 * 0.05 = 0.5
        assert!(score < 2.0);
        assert!(score > 0.0);
    }

    #[test]
    fn test_decay_formula() {
        let scorer = FrequencyScorer::new(7.0);

        // At half-life, decay should be 0.5
        let decay_at_half = scorer.calculate_decay(7.0);
        assert!((decay_at_half - 0.5).abs() < 0.01);

        // At 0 days, decay should be 1.0
        let decay_at_zero = scorer.calculate_decay(0.0);
        assert!((decay_at_zero - 1.0).abs() < 0.01);

        // At double half-life, decay should be 0.25
        let decay_at_double = scorer.calculate_decay(14.0);
        assert!((decay_at_double - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_sort_by_frequency() {
        let scorer = FrequencyScorer::new(7.0);
        let now = Utc::now().timestamp();

        let entries = vec![
            create_test_entry(1, 5, Some(now - 86400)), // 5 opens, 1 day ago
            create_test_entry(2, 10, Some(now - 86400 * 14)), // 10 opens, 14 days ago
            create_test_entry(3, 3, Some(now)), // 3 opens, just now
        ];

        let sorted = scorer.sort_by_frequency(entries);

        // Entry 1 should be highest: 5 * 0.9 ≈ 4.5
        // Entry 3 should be second: 3 * 1.0 = 3
        // Entry 2 should be lowest: 10 * 0.25 ≈ 2.5
        assert_eq!(sorted[0].id, Some(1));
        assert_eq!(sorted[1].id, Some(3));
        assert_eq!(sorted[2].id, Some(2));
    }

    #[test]
    fn test_simple_score_formula() {
        // frequency / (days + 1)
        let score_1 = FrequencyScorer::calculate_score_simple(10, 0.0);
        assert!((score_1 - 10.0).abs() < 0.01);

        let score_2 = FrequencyScorer::calculate_score_simple(10, 1.0);
        assert!((score_2 - 5.0).abs() < 0.01);

        let score_3 = FrequencyScorer::calculate_score_simple(10, 9.0);
        assert!((score_3 - 1.0).abs() < 0.01);
    }
}