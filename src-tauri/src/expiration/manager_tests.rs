//! Tests for expiration manager functionality

#[cfg(test)]
mod tests {
    use crate::expiration::{
        ExpirationConfig, ExpirationManager, ExpirationStatus,
        format_remaining_time, format_expiration_date,
    };
    use crate::models::{Entry, LnkTarget};
    use chrono::{Duration, Utc};
    use rusqlite::Connection;

    /// Create an in-memory database for testing
    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory DB");
        
        // Create the entries table
        conn.execute(
            r#"
            CREATE TABLE entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                lnk_path TEXT NOT NULL UNIQUE,
                target_path TEXT NOT NULL,
                parameters TEXT,
                working_dir TEXT,
                tags TEXT,
                notes TEXT,
                frequency INTEGER DEFAULT 0,
                last_opened INTEGER,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                expires_at INTEGER
            )
            "#,
            [],
        ).expect("Failed to create entries table");
        
        // Create index on expires_at
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_entries_expires_at ON entries(expires_at)",
            [],
        ).expect("Failed to create index");
        
        conn
    }

    /// Create a test entry
    fn create_test_entry(lnk_path: &str, target_path: &str) -> Entry {
        let now = Utc::now().timestamp();
        Entry {
            id: None,
            lnk_path: lnk_path.to_string(),
            target_path: target_path.to_string(),
            target_type: LnkTarget::File(target_path.to_string()),
            parameters: None,
            working_dir: None,
            description: None,
            icon_location: None,
            icon_index: None,
            tags: None,
            notes: None,
            frequency: 0,
            last_opened: None,
            created_at: now,
            updated_at: now,
            group_id: None,
            expires_at: None,
        }
    }

    /// Insert a test entry into the database
    fn insert_entry(conn: &Connection, entry: &Entry) -> i64 {
        conn.execute(
            r#"
            INSERT INTO entries (lnk_path, target_path, parameters, working_dir, tags, notes,
                                 frequency, last_opened, created_at, updated_at, expires_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            rusqlite::params![
                entry.lnk_path,
                entry.target_path,
                entry.parameters,
                entry.working_dir,
                entry.tags,
                entry.notes,
                entry.frequency,
                entry.last_opened,
                entry.created_at,
                entry.updated_at,
                entry.expires_at,
            ],
        ).expect("Failed to insert entry");
        
        conn.last_insert_rowid()
    }

    #[test]
    fn test_expiration_config_default() {
        let config = ExpirationConfig::default();
        assert_eq!(config.warning_days, 7);
        assert!(config.enable_notifications);
        assert!(!config.auto_delete_expired);
        assert_eq!(config.check_interval_hours, 1);
    }

    #[test]
    fn test_format_remaining_time() {
        assert_eq!(format_remaining_time(172800), "2 days"); // 2 days
        assert_eq!(format_remaining_time(86400), "1 days"); // 1 day
        assert_eq!(format_remaining_time(3600), "1 hours"); // 1 hour
        assert_eq!(format_remaining_time(7200), "2 hours"); // 2 hours
        assert_eq!(format_remaining_time(120), "2 minutes"); // 2 minutes
        assert_eq!(format_remaining_time(30), "less than a minute"); // 30 seconds
    }

    #[test]
    fn test_format_expiration_date() {
        let ts = 1704067200i64; // 2024-01-01 00:00:00 UTC
        let formatted = format_expiration_date(ts);
        assert!(formatted.contains("2024"));
        assert!(formatted.contains("01"));
    }

    #[test]
    fn test_check_expired_entries_empty() {
        let conn = create_test_db();
        let manager = ExpirationManager::new(conn);
        
        let expired = manager.check_expired_entries().expect("Failed to check expired");
        assert!(expired.is_empty());
    }

    #[test]
    fn test_check_expired_entries_with_expired() {
        let conn = create_test_db();
        
        // Insert an entry that expired yesterday
        let mut entry = create_test_entry("C:/test_expired.lnk", "C:/target.exe");
        entry.expires_at = Some((Utc::now() - Duration::days(1)).timestamp());
        insert_entry(&conn, &entry);
        
        let manager = ExpirationManager::new(conn);
        let expired = manager.check_expired_entries().expect("Failed to check expired");
        
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].lnk_path, "C:/test_expired.lnk");
    }

    #[test]
    fn test_check_expired_entries_not_yet_expired() {
        let conn = create_test_db();
        
        // Insert an entry that expires in 1 day (not yet expired)
        let mut entry = create_test_entry("C:/test_not_expired.lnk", "C:/target.exe");
        entry.expires_at = Some((Utc::now() + Duration::days(1)).timestamp());
        insert_entry(&conn, &entry);
        
        let manager = ExpirationManager::new(conn);
        let expired = manager.check_expired_entries().expect("Failed to check expired");
        
        assert!(expired.is_empty());
    }

    #[test]
    fn test_get_expiring_soon() {
        let conn = create_test_db();
        
        // Insert an entry expiring in 3 days (within warning period)
        let mut entry = create_test_entry("C:/test_expiring_soon.lnk", "C:/target.exe");
        entry.expires_at = Some((Utc::now() + Duration::days(3)).timestamp());
        insert_entry(&conn, &entry);
        
        // Insert an entry expiring in 30 days (outside warning period)
        let mut entry2 = create_test_entry("C:/test_expiring_later.lnk", "C:/target2.exe");
        entry2.expires_at = Some((Utc::now() + Duration::days(30)).timestamp());
        insert_entry(&conn, &entry2);
        
        let manager = ExpirationManager::new(conn);
        let expiring = manager.get_expiring_soon().expect("Failed to get expiring soon");
        
        assert_eq!(expiring.len(), 1);
        assert_eq!(expiring[0].0.lnk_path, "C:/test_expiring_soon.lnk");
    }

    #[test]
    fn test_set_expiration() {
        let conn = create_test_db();
        
        let entry = create_test_entry("C:/test_set_exp.lnk", "C:/target.exe");
        let id = insert_entry(&conn, &entry);
        
        let manager = ExpirationManager::new(conn);
        let expires_at = Utc::now() + Duration::days(7);
        
        manager.set_expiration(id, expires_at).expect("Failed to set expiration");
        
        // Verify the entry has expiration set
        let exp_ts: Option<i64> = conn.query_row(
            "SELECT expires_at FROM entries WHERE id = ?1",
            [id],
            |row| row.get::<_, Option<i64>>(0),
        ).expect("Failed to query");
        
        assert!(exp_ts.is_some());
        // Allow 1 second tolerance for the comparison
        assert!((exp_ts.unwrap() - expires_at.timestamp()).abs() <= 1);
    }

    #[test]
    fn test_remove_expiration() {
        let conn = create_test_db();
        
        // Insert entry with expiration
        let mut entry = create_test_entry("C:/test_remove_exp.lnk", "C:/target.exe");
        entry.expires_at = Some((Utc::now() + Duration::days(7)).timestamp());
        let id = insert_entry(&conn, &entry);
        
        let manager = ExpirationManager::new(conn);
        manager.remove_expiration(id).expect("Failed to remove expiration");
        
        // Verify expiration is removed
        let exp_ts: Option<i64> = conn.query_row(
            "SELECT expires_at FROM entries WHERE id = ?1",
            [id],
            |row| row.get::<_, Option<i64>>(0),
        ).expect("Failed to query");
        
        assert!(exp_ts.is_none());
    }

    #[test]
    fn test_extend_expiration() {
        let conn = create_test_db();
        
        // Insert entry with expiration
        let original_expiry = Utc::now() + Duration::days(5);
        let mut entry = create_test_entry("C:/test_extend_exp.lnk", "C:/target.exe");
        entry.expires_at = Some(original_expiry.timestamp());
        let id = insert_entry(&conn, &entry);
        
        let manager = ExpirationManager::new(conn);
        manager.extend_expiration(id, 7).expect("Failed to extend expiration");
        
        // Verify expiration is extended
        let new_exp_ts: i64 = conn.query_row(
            "SELECT expires_at FROM entries WHERE id = ?1",
            [id],
            |row| row.get::<_, Option<i64>>(0).map(|o| o.unwrap_or(0)),
        ).expect("Failed to query").expect("No result");
        
        // Should be extended by 7 days from original
        let expected = original_expiry.timestamp() + (7 * 86400);
        assert!((new_exp_ts - expected).abs() <= 1);
    }

    #[test]
    fn test_get_expiration_status_expired() {
        let conn = create_test_db();
        let manager = ExpirationManager::new(conn);
        
        let mut entry = create_test_entry("C:/test_status.lnk", "C:/target.exe");
        entry.expires_at = Some((Utc::now() - Duration::days(1)).timestamp());
        
        let status = manager.get_expiration_status(&entry);
        
        assert!(matches!(status, ExpirationStatus::Expired { .. }));
    }

    #[test]
    fn test_get_expiration_status_expiring_soon() {
        let conn = create_test_db();
        let manager = ExpirationManager::new(conn);
        
        let mut entry = create_test_entry("C:/test_status.lnk", "C:/target.exe");
        entry.expires_at = Some((Utc::now() + Duration::days(3)).timestamp());
        
        let status = manager.get_expiration_status(&entry);
        
        assert!(matches!(status, ExpirationStatus::ExpiringSoon { .. }));
        if let ExpirationStatus::ExpiringSoon { days_remaining, .. } = status {
            assert_eq!(days_remaining, 3);
        }
    }

    #[test]
    fn test_get_expiration_status_not_expiring() {
        let conn = create_test_db();
        let manager = ExpirationManager::new(conn);
        
        let entry = create_test_entry("C:/test_status.lnk", "C:/target.exe");
        
        let status = manager.get_expiration_status(&entry);
        
        assert!(matches!(status, ExpirationStatus::NotExpiring));
    }

    #[test]
    fn test_count_expired() {
        let conn = create_test_db();
        
        // Insert 3 expired entries
        for i in 0..3 {
            let mut entry = create_test_entry(&format!("C:/test_expired_{}.lnk", i), "C:/target.exe");
            entry.expires_at = Some((Utc::now() - Duration::days(1)).timestamp());
            insert_entry(&conn, &entry);
        }
        
        // Insert 2 non-expired entries
        for i in 0..2 {
            let mut entry = create_test_entry(&format!("C:/test_active_{}.lnk", i), "C:/target.exe");
            entry.expires_at = Some((Utc::now() + Duration::days(7)).timestamp());
            insert_entry(&conn, &entry);
        }
        
        let manager = ExpirationManager::new(conn);
        let count = manager.count_expired().expect("Failed to count");
        
        assert_eq!(count, 3);
    }

    #[test]
    fn test_delete_all_expired() {
        let conn = create_test_db();
        
        // Insert 2 expired entries
        for i in 0..2 {
            let mut entry = create_test_entry(&format!("C:/test_del_expired_{}.lnk", i), "C:/target.exe");
            entry.expires_at = Some((Utc::now() - Duration::days(1)).timestamp());
            insert_entry(&conn, &entry);
        }
        
        // Insert 1 non-expired entry
        let mut entry = create_test_entry("C:/test_del_active.lnk", "C:/target.exe");
        entry.expires_at = Some((Utc::now() + Duration::days(7)).timestamp());
        insert_entry(&conn, &entry);
        
        let manager = ExpirationManager::new(conn);
        let deleted = manager.delete_all_expired().expect("Failed to delete");

        assert_eq!(deleted, 2);

        // Verify only the non-expired entry remains
        let remaining: i64 = manager.connection().query_row(
            "SELECT COUNT(*) FROM entries",
            [],
            |row| row.get::<_, i64>(0),
        ).expect("Failed to count");
        
        assert_eq!(remaining, 1);
    }
}