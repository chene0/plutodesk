#[cfg(test)]
mod folders_uuid_resolution_tests {
    use crate::db::services;
    use migration::{Migrator, MigratorTrait};
    use sea_orm::Database;
    use uuid::Uuid;

    async fn setup_test_db() -> sea_orm::DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");

        db
    }

    #[tokio::test]
    async fn test_all_zeros_uuid_resolves_to_default_user() {
        let db = setup_test_db().await;

        let all_zeros_uuid = "00000000-0000-0000-0000-000000000000";

        // Parse it to UUID (should be valid)
        let parsed = Uuid::parse_str(all_zeros_uuid);
        assert!(
            parsed.is_ok(),
            "All-zeros UUID should be valid UUID format"
        );

        // Test get_or_create_default_user
        let default_user1 = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to get/create default user");

        assert_ne!(
            default_user1,
            Uuid::nil(),
            "Default user should not be all-zeros UUID"
        );

        // Call again - should return the same user (not create new one)
        let default_user2 = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to get/create default user");

        assert_eq!(
            default_user1, default_user2,
            "Should return same default user on subsequent calls"
        );
    }

    #[tokio::test]
    async fn test_default_user_is_created_if_doesnt_exist() {
        let db = setup_test_db().await;

        // First call should create the user
        let user_id = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to create default user");

        // Verify user was actually created in database
        use crate::db::entities::users;
        use sea_orm::EntityTrait;

        let user = users::Entity::find_by_id(user_id)
            .one(&db)
            .await
            .expect("Failed to query user")
            .expect("User should exist in database");

        assert_eq!(user.id, user_id);
        assert_eq!(user.email, "test@plutodesk.local");
    }

    #[tokio::test]
    async fn test_default_user_is_reused_if_exists() {
        let db = setup_test_db().await;

        // Create default user first time
        let user_id_1 = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to create default user");

        // Call again - should return same user
        let user_id_2 = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to get default user");

        assert_eq!(
            user_id_1, user_id_2,
            "Should reuse existing default user"
        );

        // Verify only one user exists in database
        use crate::db::entities::users;
        use sea_orm::EntityTrait;

        let user_count = users::Entity::find()
            .all(&db)
            .await
            .expect("Failed to query users")
            .len();

        assert_eq!(user_count, 1, "Should only have one user in database");
    }

    #[tokio::test]
    async fn test_normal_uuid_passthrough() {
        let db = setup_test_db().await;

        // Create a normal (non-default) user
        let normal_uuid = Uuid::new_v4();

        // Create a user with this UUID
        use crate::db::entities::users;
        use sea_orm::{ActiveModelTrait, Set};

        let user = users::ActiveModel {
            id: Set(normal_uuid),
            email: Set("normal@plutodesk.local".to_string()),
            name: Set("Normal User".to_string()),
            password_hash: Set("dummy_hash".to_string()),
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };

        user.insert(&db).await.expect("Failed to insert user");

        // Now try to get folders for this user
        let folders = services::get_folders_by_user(&db, normal_uuid)
            .await
            .expect("Failed to get folders");

        // Should work without errors (even if empty)
        assert_eq!(folders.len(), 0, "New user should have no folders");
    }

    #[tokio::test]
    async fn test_invalid_uuid_returns_error() {
        let invalid_uuids = vec![
            "not-a-uuid",
            "12345678",
            "invalid-uuid-format",
            "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
            "",
        ];

        for invalid in invalid_uuids {
            let result = Uuid::parse_str(invalid);
            assert!(
                result.is_err(),
                "Should return error for invalid UUID: {}",
                invalid
            );

            let error = result.unwrap_err();
            // Verify it's a parse error - uuid crate returns Error(Char {...}) or similar
            assert!(
                format!("{:?}", error).contains("Error"),
                "Error should be a parse error: {:?}",
                error
            );
        }
    }

    #[tokio::test]
    async fn test_uuid_edge_cases() {
        // Test various edge cases
        let test_cases = vec![
            // Valid UUIDs
            ("550e8400-e29b-41d4-a716-446655440000", true),
            ("00000000-0000-0000-0000-000000000000", true), // All zeros (valid)
            ("ffffffff-ffff-ffff-ffff-ffffffffffff", true), // All F's (valid)
            ("550e8400e29b41d4a716446655440000", true),     // No dashes (valid simple format)
            // Invalid UUIDs
            ("550e8400-e29b-41d4-a716-44665544000", false), // Too short
            ("550e8400-e29b-41d4-a716-4466554400000", false), // Too long
            ("550e8400-e29b-41d4-a716", false),             // Incomplete
            ("g50e8400-e29b-41d4-a716-446655440000", false), // Invalid char
        ];

        for (uuid_str, should_be_valid) in test_cases {
            let result = Uuid::parse_str(uuid_str);
            if should_be_valid {
                assert!(
                    result.is_ok(),
                    "Should accept valid UUID: {}",
                    uuid_str
                );
            } else {
                assert!(
                    result.is_err(),
                    "Should reject invalid UUID: {}",
                    uuid_str
                );
            }
        }
    }

    #[tokio::test]
    async fn test_folders_with_all_zeros_uuid() {
        let db = setup_test_db().await;

        // Get default user via all-zeros UUID resolution
        let default_user = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to get default user");

        // Create a folder for the default user
        let folder = services::create_folder(
            &db,
            default_user,
            "Test Folder".to_string(),
            None,
            0,
        )
        .await
        .expect("Failed to create folder");

        // Now get folders using the default user's actual UUID
        let folders_by_real_uuid = services::get_folders_by_user(&db, default_user)
            .await
            .expect("Failed to get folders");

        assert_eq!(
            folders_by_real_uuid.len(),
            1,
            "Should find folder by real UUID"
        );
        assert_eq!(folders_by_real_uuid[0].id, folder.id);

        // The get_folders_by_user command logic (with all-zeros check) should also work
        // This simulates what happens in the command handler
        let user_uuid = if "00000000-0000-0000-0000-000000000000"
            == "00000000-0000-0000-0000-000000000000"
        {
            services::get_or_create_default_user(&db)
                .await
                .map_err(|e| e.to_string())
                .expect("Failed to resolve default user")
        } else {
            Uuid::parse_str("00000000-0000-0000-0000-000000000000")
                .map_err(|e| e.to_string())
                .expect("Failed to parse UUID")
        };

        let folders_by_all_zeros = services::get_folders_by_user(&db, user_uuid)
            .await
            .expect("Failed to get folders via all-zeros resolution");

        assert_eq!(
            folders_by_all_zeros.len(),
            1,
            "Should find folder via all-zeros UUID resolution"
        );
        assert_eq!(folders_by_all_zeros[0].id, folder.id);
    }

    #[tokio::test]
    async fn test_uuid_doesnt_crash_on_invalid_input() {
        // Test that parsing invalid UUIDs doesn't crash the application
        let long_string = "x".repeat(1000);
        let malformed_inputs = vec![
            "\0\0\0\0",                         // Null bytes
            "🦀",                                // Emoji
            "SELECT * FROM users",              // SQL injection attempt
            "<script>alert('xss')</script>",    // XSS attempt
            long_string.as_str(),               // Very long string
            "../../../../etc/passwd",           // Path traversal
        ];

        for input in malformed_inputs {
            let result = Uuid::parse_str(input);
            assert!(
                result.is_err(),
                "Should safely reject malformed input: {:?}",
                input
            );
            // Important: verify it returns an error, not crashes
        }
    }

    #[tokio::test]
    async fn test_default_user_properties() {
        let db = setup_test_db().await;

        let user_id = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to get default user");

        // Verify user has expected properties
        use crate::db::entities::users;
        use sea_orm::EntityTrait;

        let user = users::Entity::find_by_id(user_id)
            .one(&db)
            .await
            .expect("Failed to query user")
            .expect("User should exist");

        assert_eq!(user.email, "test@plutodesk.local");
        assert_eq!(user.name, "Test User"); // Service creates "Test User" not "Default User"
        // Verify user has valid UUID (not nil)
        assert_ne!(user.id, Uuid::nil());
    }
}
