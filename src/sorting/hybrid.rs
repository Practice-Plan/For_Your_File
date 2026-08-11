//! Hybrid sorting algorithm
//!
//! Combines frequency, recency, and FTS score with configurable weights
//! to provide the most relevant results.

use chrono::Utc;

use super::{ScoreBreakdown, SortingWeights};
use crate::models::Entry;
use crate::search::SearchResult;

/// Hybrid scorer combining multiple factors
#[derive(Debug, Clone)]
pub struct HybridScorer {
    /// Weights for combining scores
    weights: SortingWeights,
    /// Frequency scorer (embedded)
    frequency_half_life: f32,
}

impl Default for HybridScorer {
    fn default() -> Self {
        Self::new(SortingWeights::default())
    }
}

impl HybridScorer {
    /// Create a new hybrid scorer with the given weights
    pub fn new(weights: SortingWeights) -> Self {
        Self {
            weights,
            frequency_half_life: 7.0,
        }
    }

    /// Set custom weights
    pub fn set_weights(&mut self, weights: SortingWeights) {
        if weights.validate().is_ok() {
            self.weights = weights;
        }
    }

    /// Get current weights
    pub fn weights(&self) -> &SortingWeights {
        &self.weights
    }

    /// Set frequency half-life for decay calculation
    pub fn set_frequency_half_life(&mut self, half_life: f32) {
        self.frequency_half_life = half_life;
    }

    /// Calculate frequency component score
    fn calculate_frequency_score(&self, entry: &Entry) -> f64 {
        if entry.frequency == 0 {
            return 0.0;
        }

        let frequency = entry.frequency as f64;

        // Apply time decay
        let decay = match entry.last_opened {
            Some(timestamp) => {
                let now = Utc::now().timestamp();
                let days_since = (now - timestamp) as f64 / 86400.0;
                if days_since <= 0.0 {
                    1.0
                } else {
                    0.5_f64.powf(days_since / self.frequency_half_life as f64)
                }
            }
            None => 0.0,
        };

        frequency * decay
    }

    /// Calculate recency component score
    fn calculate_recency_score(&self, entry: &Entry) -> f64 {
        match entry.last_opened {
            Some(timestamp) => {
                let now = Utc::now().timestamp();
                let hours_ago = (now - timestamp) as f64 / 3600.0;
                if hours_ago <= 0.0 {
                    1.0
                } else {
                    (-hours_ago / 24.0).exp()
                }
            }
            None => 0.0,
        }
    }

    /// Calculate hybrid score for an entry
    ///
    /// Total = w1 * normalized_frequency + w2 * normalized_recency + w3 * normalized_relevance
    pub fn calculate_score(&self, entry: &Entry, fts_score: f64) -> f64 {
        let freq_score = self.calculate_frequency_score(entry);
        let rec_score = self.calculate_recency_score(entry);

        // Combine with weights (scores are already in reasonable ranges)
        (freq_score * self.weights.frequency_weight as f64)
            + (rec_score * self.weights.recency_weight as f64)
            + (fts_score.max(0.0) * self.weights.relevance_weight as f64)
    }

    /// Calculate score with breakdown for debugging
    pub fn calculate_score_with_breakdown(
        &self,
        entry: &Entry,
        fts_score: f64,
        max_frequency: f64,
        max_recency: f64,
        max_relevance: f64,
    ) -> ScoreBreakdown {
        let freq_raw = self.calculate_frequency_score(entry);
        let rec_raw = self.calculate_recency_score(entry);

        // Normalize components
        let freq_normalized = if max_frequency > 0.0 {
            freq_raw / max_frequency
        } else {
            0.0
        };

        let rec_normalized = if max_recency > 0.0 {
            rec_raw / max_recency
        } else {
            0.0
        };

        let rel_normalized = if max_relevance > 0.0 {
            fts_score.max(0.0) / max_relevance
        } else {
            0.0
        };

        // Calculate weighted scores
        let frequency_score = freq_normalized * self.weights.frequency_weight as f64;
        let recency_score = rec_normalized * self.weights.recency_weight as f64;
        let relevance_score = rel_normalized * self.weights.relevance_weight as f64;

        let total_score = frequency_score + recency_score + relevance_score;

        ScoreBreakdown {
            frequency_score,
            recency_score,
            relevance_score,
            total_score,
        }
    }

    /// Rank search results using hybrid scoring
    pub fn rank_results(&self, mut results: Vec<SearchResult>) -> Vec<SearchResult> {
        if results.is_empty() {
            return results;
        }

        // Find max values for normalization
        let max_frequency = results
            .iter()
            .map(|r| self.calculate_frequency_score(&r.entry))
            .fold(0.0_f64, |a, b| a.max(b));

        let max_recency = results
            .iter()
            .map(|r| self.calculate_recency_score(&r.entry))
            .fold(0.0_f64, |a, b| a.max(b));

        let max_relevance = results
            .iter()
            .map(|r| r.score.abs())
            .fold(0.0_f64, |a, b| a.max(b));

        // Calculate hybrid score for each result
        for result in &mut results {
            let hybrid_score = self.calculate_score_with_breakdown(
                &result.entry,
                result.score,
                max_frequency,
                max_recency,
                max_relevance,
            );
            result.score = hybrid_score.total_score;
        }

        // Sort by hybrid score descending
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Sort entries using hybrid scoring (without FTS)
    pub fn sort_entries(&self, entries: Vec<Entry>) -> Vec<Entry> {
        if entries.is_empty() {
            return entries;
        }

        let mut scored: Vec<(Entry, f64)> = entries
            .into_iter()
            .map(|e| {
                // Without FTS, use 0 for relevance
                let score = self.calculate_score(&e, 0.0);
                (e, score)
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter().map(|(e, _)| e).collect()
    }
}

/// Builder for creating custom hybrid scorers
#[derive(Debug, Clone)]
pub struct HybridScorerBuilder {
    weights: SortingWeights,
    frequency_half_life: f32,
}

impl Default for HybridScorerBuilder {
    fn default() -> Self {
        Self {
            weights: SortingWeights::default(),
            frequency_half_life: 7.0,
        }
    }
}

impl HybridScorerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set frequency weight
    pub fn frequency_weight(mut self, weight: f32) -> Self {
        self.weights.frequency_weight = weight;
        self
    }

    /// Set recency weight
    pub fn recency_weight(mut self, weight: f32) -> Self {
        self.weights.recency_weight = weight;
        self
    }

    /// Set relevance weight
    pub fn relevance_weight(mut self, weight: f32) -> Self {
        self.weights.relevance_weight = weight;
        self
    }

    /// Set frequency half-life
    pub fn frequency_half_life(mut self, half_life: f32) -> Self {
        self.frequency_half_life = half_life;
        self
    }

    /// Build the hybrid scorer
    pub fn build(self) -> Result<HybridScorer, String> {
        self.weights.validate()?;
        let mut scorer = HybridScorer::new(self.weights);
        scorer.set_frequency_half_life(self.frequency_half_life);
        Ok(scorer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::SearchResult;

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

    fn create_test_result(id: i64, frequency: i32, last_opened: Option<i64>, score: f64) -> SearchResult {
        SearchResult {
            entry: create_test_entry(id, frequency, last_opened),
            score,
            snippet: None,
        }
    }

    #[test]
    fn test_default_weights() {
        let scorer = HybridScorer::default();
        let weights = scorer.weights();
        assert_eq!(weights.frequency_weight, 0.3);
        assert_eq!(weights.recency_weight, 0.2);
        assert_eq!(weights.relevance_weight, 0.5);
    }

    #[test]
    fn test_hybrid_score_components() {
        let weights = SortingWeights {
            frequency_weight: 0.4,
            recency_weight: 0.3,
            relevance_weight: 0.3,
        };
        let scorer = HybridScorer::new(weights);

        let now = Utc::now().timestamp();
        let entry = create_test_entry(1, 10, Some(now));

        // High frequency, recent, high relevance
        let score = scorer.calculate_score(&entry, 1.0);
        assert!(score > 0.0);
    }

    #[test]
    fn test_hybrid_score_never_opened() {
        let scorer = HybridScorer::default();
        let entry = create_test_entry(1, 0, None);

        let score = scorer.calculate_score(&entry, 0.5);
        // Should only have relevance contribution
        assert!((score - 0.5 * 0.5).abs() < 0.01);
    }

    #[test]
    fn test_rank_results() {
        let scorer = HybridScorer::default();

        let now = Utc::now().timestamp();
        let results = vec![
            create_test_result(1, 100, Some(now - 86400), 1.0), // High freq, 1 day ago
            create_test_result(2, 10, Some(now), 0.5),         // Low freq, just now
            create_test_result(3, 50, Some(now - 86400 * 7), 0.8), // Medium freq, 1 week ago
        ];

        let ranked = scorer.rank_results(results);

        // All results should be returned
        assert_eq!(ranked.len(), 3);
        // Results should be sorted by hybrid score
        for i in 1..ranked.len() {
            assert!(ranked[i - 1].score >= ranked[i].score);
        }
    }

    #[test]
    fn test_score_breakdown() {
        let scorer = HybridScorer::default();
        let now = Utc::now().timestamp();
        let entry = create_test_entry(1, 10, Some(now));

        let breakdown = scorer.calculate_score_with_breakdown(&entry, 1.0, 10.0, 1.0, 1.0);

        // All components should be non-negative
        assert!(breakdown.frequency_score >= 0.0);
        assert!(breakdown.recency_score >= 0.0);
        assert!(breakdown.relevance_score >= 0.0);
        assert!(breakdown.total_score >= 0.0);

        // Total should be sum of components
        let expected_total =
            breakdown.frequency_score + breakdown.recency_score + breakdown.relevance_score;
        assert!((breakdown.total_score - expected_total).abs() < 0.001);
    }

    #[test]
    fn test_builder() {
        let scorer = HybridScorerBuilder::new()
            .frequency_weight(0.5)
            .recency_weight(0.3)
            .relevance_weight(0.2)
            .frequency_half_life(14.0)
            .build()
            .unwrap();

        assert_eq!(scorer.weights().frequency_weight, 0.5);
        assert_eq!(scorer.weights().recency_weight, 0.3);
        assert_eq!(scorer.weights().relevance_weight, 0.2);
    }

    #[test]
    fn test_builder_invalid_weights() {
        let result = HybridScorerBuilder::new()
            .frequency_weight(0.5)
            .recency_weight(0.5)
            .relevance_weight(0.5) // Sum > 1.0
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_sort_entries() {
        let scorer = HybridScorer::default();
        let now = Utc::now().timestamp();

        let entries = vec![
            create_test_entry(1, 5, Some(now - 3600)),
            create_test_entry(2, 10, Some(now)),
            create_test_entry(3, 3, Some(now - 86400)),
        ];

        let sorted = scorer.sort_entries(entries);

        // Entry 2 should be first (highest frequency, most recent)
        assert_eq!(sorted[0].id, Some(2));
    }
}