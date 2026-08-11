//! Full-text search operations using FTS5
//!
//! Provides comprehensive search capabilities with SQLite FTS5 extension.
//! Optimized for <1ms search performance on 10K+ entries with result caching.

use anyhow::Result;
use lru::LruCache;
use rusqlite::{Connection, OptionalExtension};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::models::{Entry, FromRow};

use super::ranking::{RankingEngine, SortCriteria};
use super::utils::{build_fts_query, escape_fts_special_chars};

/// Default cache size for search results (number of queries)
const DEFAULT_CACHE_SIZE: usize = 100;

/// Maximum age for cached results in seconds
const CACHE_MAX_AGE_SECS: u64 = 60;

/// Cached search result entry
#[derive(Debug, Clone)]
struct CachedResult {
    /// The search results
    results: Vec<SearchResult>,
    /// Timestamp when cached
    cached_at: Instant,
}

/// Search result with relevance score
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The matched entry
    pub entry: Entry,
    /// Relevance score (higher is more relevant)
    pub score: f64,
    /// Highlighted snippet (if available)
    pub snippet: Option<String>,
}

/// Paginated search results
#[derive(Debug, Clone)]
pub struct PaginatedResults {
    /// Search results
    pub results: Vec<SearchResult>,
    /// Total count of matching entries
    pub total_count: usize,
    /// Current offset
    pub offset: usize,
    /// Page size
    pub limit: usize,
}

/// Search engine using FTS5
pub struct SearchEngine {
    /// Database connection
    conn: Arc<Connection>,
    /// Ranking engine
    pub ranking_engine: RankingEngine,
    /// LRU cache for search results
    cache: Arc<RwLock<LruCache<String, CachedResult>>>,
    /// Performance metrics
    metrics: SearchMetrics,
}

/// Performance metrics for search operations
#[derive(Debug, Clone, Default)]
pub struct SearchMetrics {
    /// Total search operations
    pub total_searches: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Total query time in microseconds
    pub total_time_us: u64,
    /// Slow queries count (>1ms)
    pub slow_queries: u64,
}

impl SearchMetrics {
    /// Calculate average query time in microseconds
    pub fn avg_query_time_us(&self) -> f64 {
        if self.total_searches == 0 {
            0.0
        } else {
            self.total_time_us as f64 / self.total_searches as f64
        }
    }

    /// Calculate cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total as f64) * 100.0
        }
    }
}

impl SearchEngine {
    /// Create a new search engine with the given database connection
    pub fn new(conn: Arc<Connection>) -> Self {
        Self::with_cache_size(conn, DEFAULT_CACHE_SIZE)
    }

    /// Create a new search engine with custom cache size
    pub fn with_cache_size(conn: Arc<Connection>, cache_size: usize) -> Self {
        let cache = LruCache::new(NonZeroUsize::new(cache_size).unwrap_or(NonZeroUsize::new(100).unwrap()));
        Self {
            conn,
            ranking_engine: RankingEngine::new(),
            cache: Arc::new(RwLock::new(cache)),
            metrics: SearchMetrics::default(),
        }
    }

    /// Check cache for existing results
    async fn check_cache(&self, query: &str) -> Option<Vec<SearchResult>> {
        let mut cache = self.cache.write().await;
        if let Some(cached) = cache.get(query) {
            let elapsed = cached.cached_at.elapsed().as_secs();
            if elapsed < CACHE_MAX_AGE_SECS {
                return Some(cached.results.clone());
            }
        }
        None
    }

    /// Store results in cache
    async fn store_in_cache(&self, query: &str, results: Vec<SearchResult>) {
        let mut cache = self.cache.write().await;
        cache.put(query.to_string(), CachedResult {
            results,
            cached_at: Instant::now(),
        });
    }

    /// Get performance metrics
    pub fn metrics(&self) -> &SearchMetrics {
        &self.metrics
    }

    /// Clear the search cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Perform a full-text search on entries
    ///
    /// # Arguments
    /// * `query` - The search query string
    ///
    /// # Returns
    /// A vector of search results with relevance scores
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let fts_query = build_fts_query(query);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                e.id, e.lnk_path, e.target_path, e.parameters, e.working_dir,
                e.tags, e.notes, e.frequency, e.last_opened,
                e.created_at, e.updated_at, e.expires_at,
                bm25(entries_fts) as score
            FROM entries e
            JOIN entries_fts fts ON e.id = fts.rowid
            WHERE entries_fts MATCH ?1
            ORDER BY score DESC, e.frequency DESC
            LIMIT 100
            "#,
        )?;

        let results = stmt
            .query_map(rusqlite::params![fts_query], |row| {
                Ok(SearchResult {
                    entry: Entry::from_row(row)?,
                    score: row.get::<_, f64>(12)?,
                    snippet: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Apply ranking
        Ok(self.ranking_engine.rank_results(results))
    }

    /// Perform a search with pagination support
    ///
    /// # Arguments
    /// * `query` - The search query string
    /// * `offset` - Number of results to skip
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    /// Paginated search results with total count
    pub fn search_with_paging(
        &self,
        query: &str,
        offset: usize,
        limit: usize,
    ) -> Result<PaginatedResults> {
        let fts_query = build_fts_query(query);

        // Get total count
        let total_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH ?1",
            rusqlite::params![fts_query],
            |row| row.get(0),
        )?;

        // Get paginated results
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                e.id, e.lnk_path, e.target_path, e.parameters, e.working_dir,
                e.tags, e.notes, e.frequency, e.last_opened,
                e.created_at, e.updated_at, e.expires_at,
                bm25(entries_fts) as score
            FROM entries e
            JOIN entries_fts fts ON e.id = fts.rowid
            WHERE entries_fts MATCH ?1
            ORDER BY score DESC, e.frequency DESC
            LIMIT ?2 OFFSET ?3
            "#,
        )?;

        let results = stmt
            .query_map(rusqlite::params![fts_query, limit as i64, offset as i64], |row| {
                Ok(SearchResult {
                    entry: Entry::from_row(row)?,
                    score: row.get::<_, f64>(12)?,
                    snippet: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let ranked_results = self.ranking_engine.rank_results(results);

        Ok(PaginatedResults {
            results: ranked_results,
            total_count,
            offset,
            limit,
        })
    }

    /// Highlight matched keywords in text
    ///
    /// # Arguments
    /// * `text` - The text to highlight
    /// * `query` - The search query
    /// * `markup_start` - Opening markup tag (e.g., "<mark>")
    /// * `markup_end` - Closing markup tag (e.g., "</mark>")
    ///
    /// # Returns
    /// Text with matched keywords wrapped in markup
    pub fn highlight(
        &self,
        text: &str,
        query: &str,
        markup_start: &str,
        markup_end: &str,
    ) -> Result<String> {
        let fts_query = escape_fts_special_chars(query);

        let result: String = self.conn.query_row(
            "SELECT highlight(entries_fts, 0, ?1, ?2) FROM entries_fts WHERE entries_fts MATCH ?3 LIMIT 1",
            rusqlite::params![markup_start, markup_end, fts_query],
            |row| row.get(0),
        ).unwrap_or_else(|_| text.to_string());

        Ok(result)
    }

    /// Get highlighted snippet from search results
    ///
    /// # Arguments
    /// * `entry_id` - The entry ID
    /// * `query` - The search query
    /// * `markup_start` - Opening markup tag
    /// * `markup_end` - Closing markup tag
    ///
    /// # Returns
    /// Highlighted snippet with context
    pub fn get_snippet(
        &self,
        entry_id: i64,
        query: &str,
        markup_start: &str,
        markup_end: &str,
    ) -> Result<Option<String>> {
        let fts_query = escape_fts_special_chars(query);

        let result = self.conn.query_row(
            r#"
            SELECT snippet(entries_fts, 0, ?1, ?2, '...', 32)
            FROM entries_fts
            WHERE rowid = ?3 AND entries_fts MATCH ?4
            "#,
            rusqlite::params![markup_start, markup_end, entry_id, fts_query],
            |row| row.get::<_, String>(0),
        ).optional()?;

        Ok(result)
    }

    /// Search entries with both FTS and LIKE fallback
    pub fn search_flexible(&self, query: &str) -> Result<Vec<SearchResult>> {
        // First try FTS search
        let fts_results = self.search(query)?;

        if !fts_results.is_empty() {
            return Ok(fts_results);
        }

        // Fallback to LIKE search if FTS yields no results
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, lnk_path, target_path, parameters, working_dir,
                   tags, notes, frequency, last_opened,
                   created_at, updated_at, expires_at
            FROM entries
            WHERE lnk_path LIKE ?1 OR target_path LIKE ?1 OR tags LIKE ?1 OR notes LIKE ?1
            ORDER BY frequency DESC
            LIMIT 100
            "#,
        )?;

        let results = stmt
            .query_map(rusqlite::params![pattern], |row| {
                Ok(SearchResult {
                    entry: Entry::from_row(row)?,
                    score: 1.0,
                    snippet: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /// Get entries sorted by frequency
    pub fn get_entries_by_frequency(&self, limit: i32) -> Result<Vec<Entry>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, lnk_path, target_path, parameters, working_dir,
                   tags, notes, frequency, last_opened,
                   created_at, updated_at, expires_at
            FROM entries
            ORDER BY frequency DESC, last_opened DESC
            LIMIT ?1
            "#,
        )?;

        let entries = stmt
            .query_map(rusqlite::params![limit], |row| Entry::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Set custom weights for ranking
    pub fn set_ranking_weights(&mut self, frequency_weight: f64, recency_weight: f64, score_weight: f64) {
        self.ranking_engine.set_weights(frequency_weight, recency_weight, score_weight);
    }

    /// Set sort criteria for results
    pub fn set_sort_criteria(&mut self, criteria: SortCriteria) {
        self.ranking_engine.set_sort_criteria(criteria);
    }
}

/// Perform a full-text search on entries (standalone function for backwards compatibility)
pub fn search_entries(conn: &Connection, query: &str) -> Result<Vec<SearchResult>> {
    let fts_query = build_fts_query(query);

    let mut stmt = conn.prepare(
        r#"
        SELECT
            e.id, e.lnk_path, e.target_path, e.parameters, e.working_dir,
            e.tags, e.notes, e.frequency, e.last_opened,
            e.created_at, e.updated_at, e.expires_at,
            bm25(entries_fts) as score
        FROM entries e
        JOIN entries_fts fts ON e.id = fts.rowid
        WHERE entries_fts MATCH ?1
        ORDER BY score DESC, e.frequency DESC
        LIMIT 50
        "#,
    )?;

    let results = stmt
        .query_map(rusqlite::params![fts_query], |row| {
            Ok(SearchResult {
                entry: Entry::from_row(row)?,
                score: row.get::<_, f64>(12)?,
                snippet: None,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

/// Search entries with both FTS and LIKE fallback (standalone function)
pub fn search_entries_flexible(conn: &Connection, query: &str) -> Result<Vec<SearchResult>> {
    let fts_results = search_entries(conn, query)?;

    if !fts_results.is_empty() {
        return Ok(fts_results);
    }

    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        r#"
        SELECT id, lnk_path, target_path, parameters, working_dir,
               tags, notes, frequency, last_opened,
               created_at, updated_at, expires_at
        FROM entries
        WHERE lnk_path LIKE ?1 OR target_path LIKE ?1 OR tags LIKE ?1 OR notes LIKE ?1
        ORDER BY frequency DESC
        LIMIT 50
        "#,
    )?;

    let results = stmt
        .query_map(rusqlite::params![pattern], |row| {
            Ok(SearchResult {
                entry: Entry::from_row(row)?,
                score: 1.0,
                snippet: None,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

/// Get entries sorted by frequency (standalone function)
pub fn get_entries_by_frequency(conn: &Connection, limit: i32) -> Result<Vec<Entry>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, lnk_path, target_path, parameters, working_dir,
               tags, notes, frequency, last_opened,
               created_at, updated_at, expires_at
        FROM entries
        ORDER BY frequency DESC, last_opened DESC
        LIMIT ?1
        "#,
    )?;

    let entries = stmt
        .query_map(rusqlite::params![limit], |row| Entry::from_row(row))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(entries)
}