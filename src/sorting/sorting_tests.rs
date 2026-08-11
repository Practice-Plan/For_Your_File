//! Comprehensive tests for sorting algorithms

use super::*;
use crate::models::Entry;

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

// === SortingWeights Tests ===

#[test]
fn test_default_weights_valid() {
    let weights = SortingWeights::default();
    assert!(weights.validate().is_ok());
}

#[test]
fn test_weights_new_validation() {
    // Valid weights
    let valid = SortingWeights::new(0.3, 0.3, 0.4);
    assert!(valid.is_ok());

    // Invalid weights (don't sum to 1.0)
    let invalid = SortingWeights::new(0.5, 0.5, 0.5);
    assert!(invalid.is_err());
}

#[test]
fn test_weights_normalize() {
    let mut weights = SortingWeights {
        frequency_weight: 0.5,
        recency_weight: 0.5,
        relevance_weight: 0.5,
    };

    weights.normalize();

    // After normalization, should sum to 1.0
    let sum = weights.frequency_weight + weights.recency_weight + weights.relevance_weight;
    assert!((sum - 1.0).abs() < 0.01);

    // Ratios should be preserved (all equal)
    assert!((weights.frequency_weight - 0.333).abs() < 0.01);
    assert!((weights.recency_weight - 0.333).abs() < 0.01);
    assert!((weights.relevance_weight - 0.333).abs() < 0.01);
}

#[test]
fn test_weights_setters_normalize() {
    let mut weights = SortingWeights::default();

    weights.set_frequency(0.6);

    // After setting frequency, weights should still be normalized
    let sum = weights.frequency_weight + weights.recency_weight + weights.relevance_weight;
    assert!((sum - 1.0).abs() < 0.01);
}

// === SortMethod Tests ===

#[test]
fn test_sort_method_default() {
    let method = SortMethod::default();
    assert_eq!(method, SortMethod::Relevance);
}

#[test]
fn test_sort_method_display() {
    assert_eq!(format!("{}", SortMethod::Relevance), "Relevance");
    assert_eq!(format!("{}", SortMethod::MostUsed), "Most Used");
    assert_eq!(format!("{}", SortMethod::RecentlyOpened), "Recently Opened");
    assert_eq!(format!("{}", SortMethod::Alphabetical), "Alphabetical");
    assert_eq!(format!("{}", SortMethod::Custom), "Custom");
}

// === SortingConfig Tests ===

#[test]
fn test_sorting_config_default() {
    let config = SortingConfig::default();
    assert_eq!(config.method, SortMethod::Relevance);
    assert!(!config.debug_mode);
    assert_eq!(config.frequency_half_life, 7.0);
}

// === TimeWindow Tests ===

#[test]
fn test_time_window_from_timestamp() {
    let now = chrono::Utc::now().timestamp();

    // Hour
    let window = TimeWindow::from_timestamp(now - 1800, now);
    assert_eq!(window, TimeWindow::Hour);

    // Day
    let window = TimeWindow::from_timestamp(now - 3600 * 12, now);
    assert_eq!(window, TimeWindow::Day);

    // Week
    let window = TimeWindow::from_timestamp(now - 86400 * 3, now);
    assert_eq!(window, TimeWindow::Week);

    // Month
    let window = TimeWindow::from_timestamp(now - 86400 * 15, now);
    assert_eq!(window, TimeWindow::Month);

    // Older
    let window = TimeWindow::from_timestamp(now - 86400 * 60, now);
    assert_eq!(window, TimeWindow::Older);
}

#[test]
fn test_time_window_labels() {
    assert_eq!(TimeWindow::Hour.label(), "Last Hour");
    assert_eq!(TimeWindow::Day.label(), "Today");
    assert_eq!(TimeWindow::Week.label(), "This Week");
    assert_eq!(TimeWindow::Month.label(), "This Month");
    assert_eq!(TimeWindow::Older.label(), "Older");
}

// === Frequency Scorer Tests ===

#[test]
fn test_frequency_scorer_high_frequency_recent() {
    use super::frequency::FrequencyScorer;
    use chrono::Utc;

    let scorer = FrequencyScorer::new(7.0);
    let now = Utc::now().timestamp();

    let entry = create_test_entry(1, 100, Some(now));
    let score = scorer.calculate_score(&entry);

    // High frequency, just opened - score should be close to frequency
    assert!(score > 95.0);
}

#[test]
fn test_frequency_scorer_high_frequency_old() {
    use super::frequency::FrequencyScorer;
    use chrono::Utc;

    let scorer = FrequencyScorer::new(7.0);
    let now = Utc::now().timestamp();

    // 30 days ago
    let entry = create_test_entry(1, 100, Some(now - 86400 * 30));
    let score = scorer.calculate_score(&entry);

    // High frequency, but old - score should be significantly lower
    assert!(score < 10.0);
}

#[test]
fn test_frequency_decay_calculation() {
    use super::frequency::FrequencyScorer;

    // Test simple formula
    let score_1 = FrequencyScorer::calculate_score_simple(10, 0.0);
    assert!((score_1 - 10.0).abs() < 0.01);

    let score_2 = FrequencyScorer::calculate_score_simple(10, 1.0);
    assert!((score_2 - 5.0).abs() < 0.01);

    let score_3 = FrequencyScorer::calculate_score_simple(10, 9.0);
    assert!((score_3 - 1.0).abs() < 0.01);
}

// === Recency Scorer Tests ===

#[test]
fn test_recency_score_ordering() {
    use super::recency::RecencyScorer;
    use chrono::Utc;

    let now = Utc::now().timestamp();
    let scorer = RecencyScorer::with_reference_time(now);

    let entries = vec![
        create_test_entry(1, 1, Some(now - 86400)), // 1 day ago
        create_test_entry(2, 1, Some(now - 3600)),  // 1 hour ago
        create_test_entry(3, 1, Some(now)),         // Just now
        create_test_entry(4, 1, None),              // Never opened
    ];

    let sorted = scorer.sort_by_recency(entries);

    assert_eq!(sorted[0].id, Some(3)); // Most recent
    assert_eq!(sorted[1].id, Some(2));
    assert_eq!(sorted[2].id, Some(1));
    assert_eq!(sorted[3].id, Some(4)); // Never opened at the end
}

#[test]
fn test_recency_relative_time() {
    use super::recency::RecencyScorer;
    use chrono::Utc;

    let now = Utc::now().timestamp();
    let scorer = RecencyScorer::with_reference_time(now);

    let entry_now = create_test_entry(1, 1, Some(now - 30));
    assert_eq!(scorer.get_relative_time(&entry_now), "Just now");

    let entry_min = create_test_entry(2, 1, Some(now - 300));
    assert_eq!(scorer.get_relative_time(&entry_min), "5 minutes ago");

    let entry_hour = create_test_entry(3, 1, Some(now - 7200));
    assert_eq!(scorer.get_relative_time(&entry_hour), "2 hours ago");

    let entry_day = create_test_entry(4, 1, Some(now - 172800));
    assert_eq!(scorer.get_relative_time(&entry_day), "2 days ago");
}

// === Hybrid Scorer Tests ===

#[test]
fn test_hybrid_combined_score() {
    use super::hybrid::{HybridScorer, HybridScorerBuilder};
    use chrono::Utc;

    let scorer = HybridScorerBuilder::new()
        .frequency_weight(0.4)
        .recency_weight(0.3)
        .relevance_weight(0.3)
        .build()
        .unwrap();

    let now = Utc::now().timestamp();

    // High frequency, recent, high relevance
    let entry_high = create_test_entry(1, 100, Some(now));
    let score_high = scorer.calculate_score(&entry_high, 1.0);

    // Low frequency, old, low relevance
    let entry_low = create_test_entry(2, 1, Some(now - 86400 * 30));
    let score_low = scorer.calculate_score(&entry_low, 0.1);

    assert!(score_high > score_low);
}

#[test]
fn test_hybrid_score_breakdown() {
    use super::hybrid::HybridScorer;
    use chrono::Utc;

    let scorer = HybridScorer::default();
    let now = Utc::now().timestamp();
    let entry = create_test_entry(1, 10, Some(now));

    let breakdown = scorer.calculate_score_with_breakdown(&entry, 1.0, 100.0, 1.0, 1.0);

    // All components should be in valid range
    assert!(breakdown.frequency_score >= 0.0 && breakdown.frequency_score <= 1.0);
    assert!(breakdown.recency_score >= 0.0 && breakdown.recency_score <= 1.0);
    assert!(breakdown.relevance_score >= 0.0 && breakdown.relevance_score <= 1.0);
    assert!(breakdown.total_score >= 0.0 && breakdown.total_score <= 3.0);
}

// === Performance Tests ===

#[test]
fn test_frequency_performance() {
    use super::frequency::FrequencyScorer;
    use chrono::Utc;
    use std::time::Instant;

    let scorer = FrequencyScorer::new(7.0);
    let now = Utc::now().timestamp();

    // Create 1000 test entries
    let entries: Vec<Entry> = (0..1000)
        .map(|i| create_test_entry(i, (i % 100) as i32, Some(now - i as i64 * 3600)))
        .collect();

    let start = Instant::now();
    let _scores: Vec<f64> = entries.iter().map(|e| scorer.calculate_score(e)).collect();
    let duration = start.elapsed();

    // Should complete in under 10ms for 1000 entries
    assert!(duration.as_millis() < 10, "Scoring took too long: {:?}", duration);
}

#[test]
fn test_hybrid_performance() {
    use super::hybrid::HybridScorer;
    use chrono::Utc;
    use std::time::Instant;

    let scorer = HybridScorer::default();
    let now = Utc::now().timestamp();

    // Create 1000 test entries
    let entries: Vec<Entry> = (0..1000)
        .map(|i| create_test_entry(i, (i % 100) as i32, Some(now - i as i64 * 3600)))
        .collect();

    let start = Instant::now();
    let _sorted = scorer.sort_entries(entries);
    let duration = start.elapsed();

    // Should complete in under 50ms for 1000 entries
    assert!(duration.as_millis() < 50, "Sorting took too long: {:?}", duration);
}

// === Edge Cases ===

#[test]
fn test_zero_frequency() {
    use super::frequency::FrequencyScorer;
    use chrono::Utc;

    let scorer = FrequencyScorer::default();
    let now = Utc::now().timestamp();

    let entry = create_test_entry(1, 0, Some(now));
    let score = scorer.calculate_score(&entry);

    // Zero frequency should result in zero score
    assert_eq!(score, 0.0);
}

#[test]
fn test_negative_fts_score() {
    use super::hybrid::HybridScorer;

    let scorer = HybridScorer::default();
    let entry = create_test_entry(1, 10, Some(chrono::Utc::now().timestamp()));

    // Negative FTS score should be treated as 0 for relevance
    let score = scorer.calculate_score(&entry, -1.0);
    assert!(score >= 0.0);
}

#[test]
fn test_empty_results() {
    use super::hybrid::HybridScorer;

    let scorer = HybridScorer::default();
    let results = scorer.rank_results(vec![]);

    assert!(results.is_empty());
}

#[test]
fn test_weights_boundary_values() {
    // Test with minimum values
    let min_weights = SortingWeights::new(0.0, 0.0, 1.0);
    assert!(min_weights.is_ok());

    // Test with maximum values
    let max_weights = SortingWeights::new(1.0, 0.0, 0.0);
    assert!(max_weights.is_ok());

    // Test clamping
    let weights = SortingWeights::new(2.0, -0.5, 0.5);
    assert!(weights.is_ok());
    let w = weights.unwrap();
    assert_eq!(w.frequency_weight, 1.0); // Clamped to max
    assert_eq!(w.recency_weight, 0.0);   // Clamped to min
}