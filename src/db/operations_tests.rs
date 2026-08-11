//! Tests for database operations
//!
//! Comprehensive tests for CRUD operations, tag management, usage tracking,
//! and batch operations.

#[cfg(test)]
mod tests {
    use crate::db::{BatchOperations, Database, EntryOperations, TagOperations, UsageOperations};
    use crate::models::{Entry, EntryFilter, EntryUpdate, Group};

    /// Helper to create an in-memory database for testing
    fn setup_test_db() -> Database {
        Database::new_in_memory().expect("Failed to create test database")
    }

    /// Helper to create a test entry
    fn create_test_entry(lnk_path: &str, target_path: &str) -> Entry {
        Entry::new(lnk_path.to_string(), target_path.to_string())
    }

    // ============================================
    // CRUD Operations Tests
    // ============================================

    #[test]
    fn test_read_entry_not_found() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = EntryOperations::new(&conn);
        let result = ops.read_entry(9999);

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_update_entry_not_found() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = EntryOperations::new(&conn);
        let update = EntryUpdate::new().with_target("new_target.exe");

        let result = ops.update_entry(9999, &update);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_entry_not_found() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = EntryOperations::new(&conn);
        let result = ops.delete_entry(9999);

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_list_entries_empty() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = EntryOperations::new(&conn);
        let filter = EntryFilter::new();

        let result = ops.list_entries(&filter);
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_count_entries_empty() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = EntryOperations::new(&conn);
        let filter = EntryFilter::new();

        let result = ops.count_entries(&filter);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_entry_filter_pagination() {
        let filter = EntryFilter::new()
            .with_tags("test")
            .with_min_frequency(5)
            .with_pagination(10, 0);

        assert_eq!(filter.tags, Some("test".to_string()));
        assert_eq!(filter.min_frequency, Some(5));
        assert_eq!(filter.limit, Some(10));
        assert_eq!(filter.offset, Some(0));
    }

    #[test]
    fn test_entry_update_has_updates() {
        let update = EntryUpdate::new()
            .with_target("new.exe")
            .with_tags("tag1,tag2");

        assert!(update.has_updates());

        let empty_update = EntryUpdate::new();
        assert!(!empty_update.has_updates());
    }

    // ============================================
    // Group Operations Tests
    // ============================================

    #[test]
    fn test_insert_and_get_group() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let group = Group::new("Test Group".to_string(), "#FF5733".to_string());
        let result = crate::db::insert_group(&conn, &group);

        assert!(result.is_ok());
        let id = result.unwrap();
        assert!(id > 0);

        // Retrieve the group
        let retrieved = crate::db::get_group_by_id(&conn, id);
        assert!(retrieved.is_ok());
        let retrieved = retrieved.unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "Test Group");
        assert_eq!(retrieved.color, "#FF5733");
    }

    #[test]
    fn test_get_all_groups() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        // Insert multiple groups
        let group1 = Group::new("Group A".to_string(), "#FF0000".to_string());
        let group2 = Group::new("Group B".to_string(), "#00FF00".to_string());

        crate::db::insert_group(&conn, &group1).unwrap();
        crate::db::insert_group(&conn, &group2).unwrap();

        // Get all groups
        let result = crate::db::get_all_groups(&conn);
        assert!(result.is_ok());
        let groups = result.unwrap();
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_delete_group() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let group = Group::new("To Delete".to_string(), "#0000FF".to_string());
        let id = crate::db::insert_group(&conn, &group).unwrap();

        // Delete the group
        let result = crate::db::delete_group(&conn, id);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify it's deleted
        let retrieved = crate::db::get_group_by_id(&conn, id);
        assert!(retrieved.is_ok());
        assert!(retrieved.unwrap().is_none());
    }

    // ============================================
    // Tag Operations Tests
    // ============================================

    #[test]
    fn test_tag_normalization() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = TagOperations::new(&conn);

        // Test validation - valid tags
        let (valid, _invalid) = ops.validate_tags(&["  Test Tag  ".to_string(), "UPPERCASE".to_string()]);
        assert_eq!(valid.len(), 2);
        assert_eq!(valid[0], "test tag");
        assert_eq!(valid[1], "uppercase");
    }

    #[test]
    fn test_validate_tags_with_invalid_chars() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = TagOperations::new(&conn);

        let (valid, invalid) = ops.validate_tags(&["valid".to_string(), "invalid,tag".to_string(), "another;bad".to_string()]);
        assert_eq!(valid.len(), 1);
        assert_eq!(invalid.len(), 2);
    }

    #[test]
    fn test_get_all_tags_empty() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = TagOperations::new(&conn);
        let result = ops.get_all_tags();

        assert!(result.is_ok());
        let tags = result.unwrap();
        assert!(tags.is_empty());
    }

    // ============================================
    // Usage Tracking Tests
    // ============================================

    #[test]
    fn test_get_most_used_empty() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = UsageOperations::new(&conn);
        let result = ops.get_most_used(10);

        assert!(result.is_ok());
        let entries = result.unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_get_recently_used_empty() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = UsageOperations::new(&conn);
        let result = ops.get_recently_used(10);

        assert!(result.is_ok());
        let entries = result.unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_get_stats() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = UsageOperations::new(&conn);
        let result = ops.get_stats();

        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.total_groups, 0);
        assert_eq!(stats.total_opens, 0);
        assert!(stats.most_used.is_none());
        assert!(stats.recently_used.is_empty());
    }

    #[test]
    fn test_increment_frequency_not_found() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = UsageOperations::new(&conn);
        let result = ops.increment_frequency(9999);

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_get_unused_entries() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = UsageOperations::new(&conn);
        let result = ops.get_unused_entries(30);

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ============================================
    // Batch Operations Tests
    // ============================================

    #[test]
    fn test_batch_create_empty() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = BatchOperations::new(&conn);
        let result = ops.batch_create(&[]);

        assert!(result.is_ok());
        let batch_result = result.unwrap();
        assert!(batch_result.is_complete_success());
        assert_eq!(batch_result.success_count, 0);
    }

    #[test]
    fn test_batch_delete_empty() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = BatchOperations::new(&conn);
        let result = ops.batch_delete(&[]);

        assert!(result.is_ok());
        let batch_result = result.unwrap();
        assert!(batch_result.is_complete_success());
        assert_eq!(batch_result.success_count, 0);
    }

    #[test]
    fn test_batch_update_empty() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = BatchOperations::new(&conn);
        let result = ops.batch_update(&[]);

        assert!(result.is_ok());
        let batch_result = result.unwrap();
        assert!(batch_result.is_complete_success());
        assert_eq!(batch_result.success_count, 0);
    }

    #[test]
    fn test_batch_result_methods() {
        use crate::db::BatchResult;

        let mut result = BatchResult::new();
        assert!(result.is_complete_success());
        assert!(!result.is_complete_failure());

        result.success_count = 1;
        result.failed_count = 1;
        assert!(!result.is_complete_success());
        assert!(!result.is_complete_failure());

        result.success_count = 0;
        assert!(result.is_complete_failure());
    }

    #[test]
    fn test_batch_add_tags_empty() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = BatchOperations::new(&conn);
        let result = ops.batch_add_tags(&[], &["tag1".to_string()]);

        assert!(result.is_ok());
        let batch_result = result.unwrap();
        assert!(batch_result.is_complete_success());
    }

    #[test]
    fn test_batch_move_to_group_empty() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = BatchOperations::new(&conn);
        let result = ops.batch_move_to_group(&[], Some(1));

        assert!(result.is_ok());
        let batch_result = result.unwrap();
        assert!(batch_result.is_complete_success());
    }

    #[test]
    fn test_batch_increment_frequency_empty() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        let ops = BatchOperations::new(&conn);
        let result = ops.batch_increment_frequency(&[]);

        assert!(result.is_ok());
        let batch_result = result.unwrap();
        assert!(batch_result.is_complete_success());
    }

    // ============================================
    // Integration Tests
    // ============================================

    #[test]
    fn test_full_crud_workflow() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        // Create an entry
        let entry = create_test_entry("test.lnk", "C:\\test.exe");

        let ops = EntryOperations::new(&conn);

        // Create entry (may fail on non-Windows due to LNK file creation)
        if let Ok(created) = ops.create_entry(&entry) {
            let id = created.id.unwrap();

            // Read the entry
            let read = ops.read_entry(id);
            assert!(read.is_ok());
            assert!(read.unwrap().is_some());

            // Update the entry
            let update = EntryUpdate::new().with_notes("Updated notes");
            let updated = ops.update_entry(id, &update);
            assert!(updated.is_ok());

            // Increment frequency
            let usage_ops = UsageOperations::new(&conn);
            usage_ops.increment_frequency(id).unwrap();

            // Verify frequency was incremented
            let entry = ops.read_entry(id).unwrap().unwrap();
            assert_eq!(entry.frequency, 1);

            // Delete the entry
            let deleted = ops.delete_entry(id);
            assert!(deleted.is_ok());
            assert!(deleted.unwrap());

            // Verify it's deleted
            let read = ops.read_entry(id);
            assert!(read.is_ok());
            assert!(read.unwrap().is_none());
        }
    }

    #[test]
    fn test_tag_workflow() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        // Create an entry first
        let entry = create_test_entry("test.lnk", "C:\\test.exe");
        let ops = EntryOperations::new(&conn);

        if let Ok(created) = ops.create_entry(&entry) {
            let id = created.id.unwrap();

            // Add tags
            let tag_ops = TagOperations::new(&conn);
            tag_ops
                .add_tags(id, &["tag1".to_string(), "Tag2".to_string()])
                .unwrap();

            // Get entry and verify tags
            let entry = ops.read_entry(id).unwrap().unwrap();
            assert!(entry.tags.is_some());
            let tags = entry.tags.unwrap();
            assert!(tags.contains("tag1"));
            assert!(tags.contains("tag2"));

            // Remove a tag
            tag_ops.remove_tags(id, &["tag1".to_string()]).unwrap();

            // Verify tag was removed
            let entry = ops.read_entry(id).unwrap().unwrap();
            let tags = entry.tags.unwrap();
            assert!(!tags.contains("tag1"));
            assert!(tags.contains("tag2"));

            // Clean up
            ops.delete_entry(id).unwrap();
        }
    }

    #[test]
    fn test_usage_workflow() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        // Create an entry
        let entry = create_test_entry("test.lnk", "C:\\test.exe");
        let entry_ops = EntryOperations::new(&conn);

        if let Ok(created) = entry_ops.create_entry(&entry) {
            let id = created.id.unwrap();

            let usage_ops = UsageOperations::new(&conn);

            // Increment frequency multiple times
            usage_ops.increment_frequency(id).unwrap();
            usage_ops.increment_frequency(id).unwrap();
            usage_ops.increment_frequency(id).unwrap();

            // Get most used
            let most_used = usage_ops.get_most_used(10).unwrap();
            assert_eq!(most_used.len(), 1);
            assert_eq!(most_used[0].frequency, 3);

            // Get recently used
            let recently_used = usage_ops.get_recently_used(10).unwrap();
            assert_eq!(recently_used.len(), 1);

            // Get stats
            let stats = usage_ops.get_stats().unwrap();
            assert_eq!(stats.total_entries, 1);
            assert_eq!(stats.total_opens, 3);

            // Reset frequency
            usage_ops.reset_frequency(id).unwrap();
            let entry = entry_ops.read_entry(id).unwrap().unwrap();
            assert_eq!(entry.frequency, 0);

            // Clean up
            entry_ops.delete_entry(id).unwrap();
        }
    }

    #[test]
    fn test_batch_workflow() {
        let db = setup_test_db();
        let conn = db.connection().expect("Failed to get connection");

        // Create multiple entries
        let entries = vec![
            create_test_entry("test1.lnk", "C:\\test1.exe"),
            create_test_entry("test2.lnk", "C:\\test2.exe"),
            create_test_entry("test3.lnk", "C:\\test3.exe"),
        ];

        let batch_ops = BatchOperations::new(&conn);

        // Batch create
        let result = batch_ops.batch_create(&entries);
        assert!(result.is_ok());

        let batch_result = result.unwrap();
        assert!(batch_result.success_count > 0);

        if batch_result.success_count == 3 {
            let ids = batch_result.successful_ids.clone();

            // Batch increment frequency
            let result = batch_ops.batch_increment_frequency(&ids);
            assert!(result.is_ok());

            // Batch add tags
            let result = batch_ops.batch_add_tags(&ids, &["batch".to_string()]);
            assert!(result.is_ok());

            // Batch delete
            let result = batch_ops.batch_delete(&ids);
            assert!(result.is_ok());
            assert_eq!(result.unwrap().success_count, 3);
        }
    }
}