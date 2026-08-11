//! Search result ranking engine
//!
//! Provides intelligent ranking of search results based on relevance,
//! usage frequency, and time decay.

use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::fts::SearchResult;
use crate::sorting::{
    FrequencyScorer, HybridScorer, RecencyScorer, ScoreBreakdown, SortMethod, SortingConfig,
    SortingWeights, TimeWindow,
};

/// Sort criteria for search results (legacy compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortCriteria {
    /// Sort by relevance score (default)
    Relevance,
    /// Sort by usage frequency
    Frequency,
    /// Sort by recency (last opened)
    Recency,
    /// Sort by creation date
    CreationDate,
    /// Sort alphabetically
    Alphabetical,
    /// Sort with custom weights
    Custom,
}

impl Default for SortCriteria {
    fn default() -> Self {
        Self::Relevance
    }
}

impl From<SortMethod> for SortCriteria {
    fn from(method: SortMethod) -> Self {
        match method {
            SortMethod::Relevance => SortCriteria::Relevance,
            SortMethod::MostUsed => SortCriteria::Frequency,
            SortMethod::RecentlyOpened => SortCriteria::Recency,
            SortMethod::Alphabetical => SortCriteria::Alphabetical,
            SortMethod::Custom => SortCriteria::Custom,
        }
    }
}

impl From<SortCriteria> for SortMethod {
    fn from(criteria: SortCriteria) -> Self {
        match criteria {
            SortCriteria::Relevance => SortMethod::Relevance,
            SortCriteria::Frequency => SortMethod::MostUsed,
            SortCriteria::Recency => SortMethod::RecentlyOpened,
            SortCriteria::CreationDate => SortMethod::RecentlyOpened, // Fallback
            SortCriteria::Alphabetical => SortMethod::Alphabetical,
            SortCriteria::Custom => SortMethod::Custom,
        }
    }
}

/// Ranking engine for search results
#[derive(Debug, Clone)]
pub struct RankingEngine {
    /// Sorting configuration
    config: SortingConfig,
    /// Frequency scorer
    frequency_scorer: FrequencyScorer,
    /// Recency scorer
    recency_scorer: RecencyScorer,
    /// Hybrid scorer for combined ranking
    hybrid_scorer: HybridScorer,
}

impl Default for RankingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RankingEngine {
    /// Create a new ranking engine with default weights
    pub fn new() -> Self {
        let config = SortingConfig::default();
        Self::with_config(config)
    }

    /// Create a ranking engine with custom configuration
    pub fn with_config(config: SortingConfig) -> Self {
        let frequency_scorer = FrequencyScorer::new(config.frequency_half_life);
        let recency_scorer = RecencyScorer::new();
        let hybrid_scorer = HybridScorer::new(config.weights.clone());

        Self {
            config,
            frequency_scorer,
            recency_scorer,
            hybrid_scorer,
        }
    }

    /// Set custom weights for ranking
    ///
    /// # Arguments
    /// * `frequency_weight` - Weight for usage frequency
    /// * `recency_weight` - Weight for recency
    /// * `score_weight` - Weight for FTS relevance score
    pub fn set_weights(&mut self, frequency_weight: f64, recency_weight: f64, score_weight: f64) {
        let weights = SortingWeights {
            frequency_weight: frequency_weight as f32,
            recency_weight: recency_weight as f32,
            relevance_weight: score_weight as f32,
        };
        if weights.validate().is_ok() {
            self.config.weights = weights;
            self.hybrid_scorer = HybridScorer::new(self.config.weights.clone());
        }
    }

    /// Set sorting weights from SortingWeights struct
    pub fn set_sorting_weights(&mut self, weights: SortingWeights) {
        if weights.validate().is_ok() {
            self.config.weights = weights;
            self.hybrid_scorer = HybridScorer::new(self.config.weights.clone());
        }
    }

    /// Set sort criteria for results
    pub fn set_sort_criteria(&mut self, criteria: SortCriteria) {
        self.config.method = criteria.into();
    }

    /// Set sort method for results
    pub fn set_sort_method(&mut self, method: SortMethod) {
        self.config.method = method;
    }

    /// Get current sort criteria
    pub fn get_sort_criteria(&self) -> SortCriteria {
        self.config.method.into()
    }

    /// Get current sort method
    pub fn get_sort_method(&self) -> SortMethod {
        self.config.method
    }

    /// Get current configuration
    pub fn config(&self) -> &SortingConfig {
        &self.config
    }

    /// Get mutable configuration
    pub fn config_mut(&mut self) -> &mut SortingConfig {
        &mut self.config
    }

    /// Set frequency half-life for decay calculation
    pub fn set_frequency_half_life(&mut self, half_life: f32) {
        self.config.frequency_half_life = half_life;
        self.frequency_scorer = FrequencyScorer::new(half_life);
        self.hybrid_scorer.set_frequency_half_life(half_life);
    }

    /// Enable or disable debug mode
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.config.debug_mode = enabled;
    }

    /// Calculate time decay factor based on last opened time
    ///
    /// More recent usage gets higher weight
    fn calculate_time_decay(&self, last_opened: Option<i64>) -> f64 {
        match last_opened {
            Some(timestamp) => {
                let now = Utc::now().timestamp();
                let days_since_opened = (now - timestamp) as f64 / 86400.0;

                // Exponential decay: more recent = higher value
                0.95_f64.powf(days_since_opened)
            }
            None => 0.0, // Never opened
        }
    }

    /// Normalize a value to 0.0 - 1.0 range
    fn normalize(&self, value: f64, max_value: f64) -> f64 {
        if max_value <= 0.0 {
            0.0
        } else {
            (value / max_value).min(1.0)
        }
    }

    /// Get score breakdown for debug display
    pub fn get_score_breakdown(&self, result: &SearchResult, max_values: (f64, f64, f64)) -> ScoreBreakdown {
        self.hybrid_scorer.calculate_score_with_breakdown(
            &result.entry,
            result.score,
            max_values.0,
            max_values.1,
            max_values.2,
        )
    }

    /// Rank and sort search results
    ///
    /// # Arguments
    /// * `results` - Search results to rank
    ///
    /// # Returns
    /// Ranked and sorted search results
    pub fn rank_results(&self, mut results: Vec<SearchResult>) -> Vec<SearchResult> {
        if results.is_empty() {
            return results;
        }

        // Sort based on current method
        match self.config.method {
            SortMethod::Relevance => {
                // Use hybrid scorer for relevance ranking
                return self.hybrid_scorer.rank_results(results);
            }
            SortMethod::MostUsed => {
                // Sort by frequency with time decay
                results.sort_by(|a, b| {
                    let score_a = self.frequency_scorer.calculate_score(&a.entry);
                    let score_b = self.frequency_scorer.calculate_score(&b.entry);
                    score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            SortMethod::RecentlyOpened => {
                // Sort by last opened timestamp
                results.sort_by(|a, b| {
                    match (a.entry.last_opened, b.entry.last_opened) {
                        (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                });
            }
            SortMethod::Alphabetical => {
                // Sort alphabetically by filename
                results.sort_by(|a, b| {
                    a.entry.lnk_path.cmp(&b.entry.lnk_path)
                });
            }
            SortMethod::Custom => {
                // Use hybrid scorer with custom weights
                return self.hybrid_scorer.rank_results(results);
            }
        }

        results
    }

    /// Rank results and return with score breakdowns (for debug mode)
    pub fn rank_results_with_breakdown(&self, results: Vec<SearchResult>) -> Vec<(SearchResult, ScoreBreakdown)> {
        if results.is_empty() {
            return vec![];
        }

        // Calculate max values for normalization
        let max_frequency = results
            .iter()
            .map(|r| self.frequency_scorer.calculate_score(&r.entry))
            .fold(0.0_f64, |a, b| a.max(b));

        let max_recency = results
            .iter()
            .map(|r| self.recency_scorer.calculate_score(&r.entry))
            .fold(0.0_f64, |a, b| a.max(b));

        let max_relevance = results
            .iter()
            .map(|r| r.score.abs())
            .fold(0.0_f64, |a, b| a.max(b));

        // Rank results
        let ranked = self.rank_results(results);

        // Calculate breakdowns for each result
        ranked
            .into_iter()
            .map(|result| {
                let breakdown = self.get_score_breakdown(
                    &result,
                    (max_frequency, max_recency, max_relevance),
                );
                (result, breakdown)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Entry, LnkTarget};

    fn create_test_entry(id: i64, frequency: i32, last_opened: Option<i64>) -> SearchResult {
        SearchResult {
            entry: Entry {
                id: Some(id),
                lnk_path: format!("test_{}.lnk", id),
                target_path: format!("target_{}.exe", id),
                target_type: LnkTarget::File(format!("target_{}.exe", id)),
                parameters: None,
                working_dir: None,
                description: None,
                icon_location: None,
                icon_index: None,
                tags: None,
                notes: None,
                frequency,
                last_opened,
                created_at: 0,
                updated_at: 0,
                group_id: None,
                expires_at: None,
            },
            score: 1.0,
            snippet: None,
        }
    }

    #[test]
    fn test_ranking_engine_creation() {
        let engine = RankingEngine::new();
        // Test that engine can be created successfully
        assert!(matches!(engine.get_sort_criteria(), SortCriteria::Relevance));
    }

    #[test]
    fn test_set_weights() {
        let mut engine = RankingEngine::new();
        engine.set_weights(0.4, 0.3, 0.3);
        // Test that weights can be set without error
        // No direct way to verify since fields are private
    }

    #[test]
    fn test_rank_results_frequency() {
        let mut engine = RankingEngine::new();
        engine.set_sort_criteria(SortCriteria::Frequency);

        let results = vec![
            create_test_entry(1, 5, None),
            create_test_entry(2, 10, None),
            create_test_entry(3, 3, None),
        ];

        let ranked = engine.rank_results(results);
        
        assert_eq!(ranked[0].entry.id, Some(2)); // highest frequency
        assert_eq!(ranked[1].entry.id, Some(1));
        assert_eq!(ranked[2].entry.id, Some(3));
    }

    #[test]
    fn test_time_decay() {
        let engine = RankingEngine::new();
        
        // Recent usage should have higher decay
        let recent = Utc::now().timestamp() - 3600; // 1 hour ago
        let old = Utc::now().timestamp() - 86400 * 30; // 30 days ago
        
        let recent_decay = engine.calculate_time_decay(Some(recent));
        let old_decay = engine.calculate_time_decay(Some(old));
        
        assert!(recent_decay > old_decay);
    }

    #[test]
    fn test_normalize() {
        let engine = RankingEngine::new();
        
        assert_eq!(engine.normalize(5.0, 10.0), 0.5);
        assert_eq!(engine.normalize(10.0, 10.0), 1.0);
        assert_eq!(engine.normalize(15.0, 10.0), 1.0); // capped at 1.0
        assert_eq!(engine.normalize(0.0, 10.0), 0.0);
        assert_eq!(engine.normalize(5.0, 0.0), 0.0); // handle zero max
    }
}