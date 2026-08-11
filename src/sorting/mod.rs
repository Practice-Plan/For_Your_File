//! Sorting algorithms module
//!
//! Provides intelligent sorting algorithms combining frequency, recency,
//! and custom weights for LNK file management.

mod frequency;
mod hybrid;
mod recency;

pub use frequency::*;
pub use hybrid::*;
pub use recency::*;

use serde::{Deserialize, Serialize};

/// Sort method for search results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortMethod {
    /// Sort by relevance (FTS score + usage)
    Relevance,
    /// Sort by usage frequency with time decay
    MostUsed,
    /// Sort by last opened timestamp
    RecentlyOpened,
    /// Sort alphabetically by filename/path
    Alphabetical,
    /// Sort with custom weights
    Custom,
}

impl Default for SortMethod {
    fn default() -> Self {
        Self::Relevance
    }
}

impl std::fmt::Display for SortMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortMethod::Relevance => write!(f, "Relevance"),
            SortMethod::MostUsed => write!(f, "Most Used"),
            SortMethod::RecentlyOpened => write!(f, "Recently Opened"),
            SortMethod::Alphabetical => write!(f, "Alphabetical"),
            SortMethod::Custom => write!(f, "Custom"),
        }
    }
}

/// User-defined sorting weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortingWeights {
    /// Weight for frequency factor (0.0 - 1.0)
    pub frequency_weight: f32,
    /// Weight for recency factor (0.0 - 1.0)
    pub recency_weight: f32,
    /// Weight for relevance/FTS score factor (0.0 - 1.0)
    pub relevance_weight: f32,
}

impl Default for SortingWeights {
    fn default() -> Self {
        Self {
            frequency_weight: 0.3,
            recency_weight: 0.2,
            relevance_weight: 0.5,
        }
    }
}

impl SortingWeights {
    /// Create new sorting weights with validation
    pub fn new(frequency: f32, recency: f32, relevance: f32) -> Result<Self, String> {
        let weights = Self {
            frequency_weight: frequency.clamp(0.0, 1.0),
            recency_weight: recency.clamp(0.0, 1.0),
            relevance_weight: relevance.clamp(0.0, 1.0),
        };

        weights.validate()?;
        Ok(weights)
    }

    /// Validate that weights sum to approximately 1.0
    pub fn validate(&self) -> Result<(), String> {
        let sum = self.frequency_weight + self.recency_weight + self.relevance_weight;
        if (sum - 1.0).abs() > 0.01 {
            return Err(format!(
                "Weights must sum to 1.0, got {:.2}",
                sum
            ));
        }
        Ok(())
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize(&mut self) {
        let sum = self.frequency_weight + self.recency_weight + self.relevance_weight;
        if sum > 0.0 {
            self.frequency_weight /= sum;
            self.recency_weight /= sum;
            self.relevance_weight /= sum;
        }
    }

    /// Set frequency weight (automatically normalizes)
    pub fn set_frequency(&mut self, value: f32) {
        self.frequency_weight = value.clamp(0.0, 1.0);
        self.normalize();
    }

    /// Set recency weight (automatically normalizes)
    pub fn set_recency(&mut self, value: f32) {
        self.recency_weight = value.clamp(0.0, 1.0);
        self.normalize();
    }

    /// Set relevance weight (automatically normalizes)
    pub fn set_relevance(&mut self, value: f32) {
        self.relevance_weight = value.clamp(0.0, 1.0);
        self.normalize();
    }
}

/// Configuration for sorting behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortingConfig {
    /// Current sort method
    pub method: SortMethod,
    /// Custom weights (used when method is Custom)
    pub weights: SortingWeights,
    /// Half-life for frequency decay in days
    pub frequency_half_life: f32,
    /// Enable debug mode (show score breakdown)
    #[serde(default)]
    pub debug_mode: bool,
}

impl Default for SortingConfig {
    fn default() -> Self {
        Self {
            method: SortMethod::default(),
            weights: SortingWeights::default(),
            frequency_half_life: 7.0, // 7 days half-life
            debug_mode: false,
        }
    }
}

/// Time window for grouping results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeWindow {
    /// Within the last hour
    Hour,
    /// Within the last day
    Day,
    /// Within the last week
    Week,
    /// Within the last month
    Month,
    /// Older than a month
    Older,
}

impl TimeWindow {
    /// Get the time window for a given timestamp
    pub fn from_timestamp(timestamp: i64, now: i64) -> Self {
        let seconds_diff = now - timestamp;
        let hours_diff = seconds_diff / 3600;

        if hours_diff < 1 {
            TimeWindow::Hour
        } else if hours_diff < 24 {
            TimeWindow::Day
        } else if hours_diff < 24 * 7 {
            TimeWindow::Week
        } else if hours_diff < 24 * 30 {
            TimeWindow::Month
        } else {
            TimeWindow::Older
        }
    }

    /// Get a display label for the time window
    pub fn label(&self) -> &'static str {
        match self {
            TimeWindow::Hour => "Last Hour",
            TimeWindow::Day => "Today",
            TimeWindow::Week => "This Week",
            TimeWindow::Month => "This Month",
            TimeWindow::Older => "Older",
        }
    }
}

/// Score breakdown for debug display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Frequency component score
    pub frequency_score: f64,
    /// Recency component score
    pub recency_score: f64,
    /// Relevance/FTS component score
    pub relevance_score: f64,
    /// Total combined score
    pub total_score: f64,
}

#[cfg(test)]
mod sorting_tests;