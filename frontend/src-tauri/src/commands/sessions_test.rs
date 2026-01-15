#[cfg(test)]
mod session_commands_tests {
    use crate::commands::sessions::{CreateSessionRequest, SessionResponse};
    use crate::db::services;
    use crate::session::SessionManager;
    use migration::{Migrator, MigratorTrait};
    use sea_orm::Database;
    use tempfile::tempdir;

    async fn setup_test_db() -> sea_orm::DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");

        db
    }

    // Helper to create a mock AppHandle for testing
    // Note: This is simplified - in real tests you'd use Tauri's test utilities
    fn get_test_sessions_path() -> std::path::PathBuf {
        let dir = tempdir().unwrap();
        dir.path().join("test_sessions.json")
    }

    #[tokio::test]
    async fn test_create_and_start_session_new_entities() {
        let db = setup_test_db().await;

        // Get or create default user
        let user_id = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to create user");

        // Create session request
        let request = CreateSessionRequest {
            folder_name: "Computer Science".to_string(),
            course_name: "Algorithms".to_string(),
            subject_name: "Dynamic Programming".to_string(),
        };

        // Manually execute the session creation logic (without AppHandle)
        let folder = services::find_or_create_folder(&db, user_id, request.folder_name.clone())
            .await
            .expect("Failed to create folder");

        let course =
            services::find_or_create_course(&db, folder.id, request.course_name.clone())
                .await
                .expect("Failed to create course");

        let subject =
            services::find_or_create_subject(&db, course.id, request.subject_name.clone())
                .await
                .expect("Failed to create subject");

        let session_name = format!(
            "{} / {} / {}",
            request.folder_name, request.course_name, request.subject_name
        );

        let mut session_manager = SessionManager::new();
        let session = session_manager.create_session(
            session_name.clone(),
            folder.id,
            course.id,
            subject.id,
            true,
        );

        // Verify session was created correctly
        assert_eq!(session.name, session_name);
        assert_eq!(session.folder_id, folder.id);
        assert_eq!(session.course_id, course.id);
        assert_eq!(session.subject_id, subject.id);

        // Verify session is active
        assert_eq!(session_manager.active_session_id, Some(session.id));
        assert_eq!(session_manager.sessions.len(), 1);
    }

    #[tokio::test]
    async fn test_create_and_start_session_reuses_existing_entities() {
        let db = setup_test_db().await;

        let user_id = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to create user");

        // Create entities first
        let folder1 = services::find_or_create_folder(&db, user_id, "Mathematics".to_string())
            .await
            .expect("Failed to create folder");

        let course1 = services::find_or_create_course(&db, folder1.id, "Calculus".to_string())
            .await
            .expect("Failed to create course");

        let subject1 =
            services::find_or_create_subject(&db, course1.id, "Derivatives".to_string())
                .await
                .expect("Failed to create subject");

        // Try to create again with same names
        let folder2 = services::find_or_create_folder(&db, user_id, "Mathematics".to_string())
            .await
            .expect("Failed to find/create folder");

        let course2 = services::find_or_create_course(&db, folder2.id, "Calculus".to_string())
            .await
            .expect("Failed to find/create course");

        let subject2 =
            services::find_or_create_subject(&db, course2.id, "Derivatives".to_string())
                .await
                .expect("Failed to find/create subject");

        // Should have reused the same entities
        assert_eq!(folder1.id, folder2.id);
        assert_eq!(course1.id, course2.id);
        assert_eq!(subject1.id, subject2.id);
    }

    #[tokio::test]
    async fn test_duplicate_session_prevention() {
        let db = setup_test_db().await;

        let user_id = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to create user");

        let folder = services::find_or_create_folder(&db, user_id, "Physics".to_string())
            .await
            .expect("Failed to create folder");

        let course = services::find_or_create_course(&db, folder.id, "Mechanics".to_string())
            .await
            .expect("Failed to create course");

        let subject = services::find_or_create_subject(&db, course.id, "Kinematics".to_string())
            .await
            .expect("Failed to create subject");

        let mut session_manager = SessionManager::new();

        // Create first session
        session_manager.create_session(
            "Physics / Mechanics / Kinematics".to_string(),
            folder.id,
            course.id,
            subject.id,
            true,
        );

        // Try to create duplicate session - should detect it exists
        let duplicate_exists =
            session_manager.session_exists_for_context(folder.id, course.id, subject.id);
        assert!(
            duplicate_exists,
            "Should detect duplicate session with same folder/course/subject"
        );
    }

    #[tokio::test]
    async fn test_start_session_updates_timestamp() {
        let mut session_manager = SessionManager::new();
        let folder_id = uuid::Uuid::new_v4();
        let course_id = uuid::Uuid::new_v4();
        let subject_id = uuid::Uuid::new_v4();

        let session = session_manager.create_session(
            "Test Session".to_string(),
            folder_id,
            course_id,
            subject_id,
            false,
        );

        let initial_last_used = session.last_used;

        // Wait a bit to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Start the session
        session_manager
            .start_session(session.id)
            .expect("Failed to start session");

        // Verify last_used was updated
        let updated_session = session_manager.get_active_session().unwrap();
        assert!(
            updated_session.last_used > initial_last_used,
            "last_used timestamp should be updated"
        );
        assert_eq!(
            session_manager.active_session_id,
            Some(session.id),
            "Session should be active"
        );
    }

    #[tokio::test]
    async fn test_start_session_nonexistent_id() {
        let mut session_manager = SessionManager::new();
        let nonexistent_id = uuid::Uuid::new_v4();

        let result = session_manager.start_session(nonexistent_id);
        assert!(result.is_err(), "Should return error for nonexistent ID");
        assert!(result
            .unwrap_err()
            .contains("not found"));
    }

    #[tokio::test]
    async fn test_end_session() {
        let mut session_manager = SessionManager::new();
        let folder_id = uuid::Uuid::new_v4();
        let course_id = uuid::Uuid::new_v4();
        let subject_id = uuid::Uuid::new_v4();

        let session = session_manager.create_session(
            "Test Session".to_string(),
            folder_id,
            course_id,
            subject_id,
            true,
        );

        // Verify session is active
        assert!(session_manager.get_active_session().is_some());

        // End the session
        session_manager.end_session();

        // Verify session is no longer active
        assert!(session_manager.get_active_session().is_none());

        // Verify session still exists in saved sessions
        assert_eq!(session_manager.sessions.len(), 1);
        assert_eq!(session_manager.sessions[0].id, session.id);
    }

    #[tokio::test]
    async fn test_delete_session() {
        let mut session_manager = SessionManager::new();
        let folder_id = uuid::Uuid::new_v4();
        let course_id = uuid::Uuid::new_v4();
        let subject_id = uuid::Uuid::new_v4();

        let session = session_manager.create_session(
            "Test Session".to_string(),
            folder_id,
            course_id,
            subject_id,
            false,
        );

        // Verify session exists
        assert_eq!(session_manager.sessions.len(), 1);

        // Delete the session
        session_manager
            .delete_session(session.id)
            .expect("Failed to delete session");

        // Verify session was removed
        assert_eq!(session_manager.sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_active_session_clears_active_id() {
        let mut session_manager = SessionManager::new();
        let folder_id = uuid::Uuid::new_v4();
        let course_id = uuid::Uuid::new_v4();
        let subject_id = uuid::Uuid::new_v4();

        let session = session_manager.create_session(
            "Test Session".to_string(),
            folder_id,
            course_id,
            subject_id,
            true, // Start immediately
        );

        // Verify session is active
        assert!(session_manager.get_active_session().is_some());
        assert_eq!(session_manager.active_session_id, Some(session.id));

        // Delete the active session
        session_manager
            .delete_session(session.id)
            .expect("Failed to delete session");

        // Verify active_session_id was cleared
        assert!(session_manager.get_active_session().is_none());
        assert_eq!(session_manager.active_session_id, None);
    }

    #[tokio::test]
    async fn test_delete_session_nonexistent_id() {
        let mut session_manager = SessionManager::new();
        let nonexistent_id = uuid::Uuid::new_v4();

        let result = session_manager.delete_session(nonexistent_id);
        assert!(result.is_err(), "Should return error for nonexistent ID");
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn test_get_all_sessions_empty() {
        let session_manager = SessionManager::new();
        let sessions = session_manager.get_all_sessions();
        assert_eq!(sessions.len(), 0, "Should return empty array");
    }

    #[tokio::test]
    async fn test_get_all_sessions_multiple() {
        let mut session_manager = SessionManager::new();

        // Create multiple sessions
        for i in 1..=3 {
            let folder_id = uuid::Uuid::new_v4();
            let course_id = uuid::Uuid::new_v4();
            let subject_id = uuid::Uuid::new_v4();
            session_manager.create_session(
                format!("Session {}", i),
                folder_id,
                course_id,
                subject_id,
                false,
            );
        }

        let sessions = session_manager.get_all_sessions();
        assert_eq!(sessions.len(), 3, "Should return all sessions");
    }

    #[tokio::test]
    async fn test_get_active_session_none() {
        let session_manager = SessionManager::new();
        assert!(
            session_manager.get_active_session().is_none(),
            "Should return None when no active session"
        );
    }

    #[tokio::test]
    async fn test_get_active_session_some() {
        let mut session_manager = SessionManager::new();
        let folder_id = uuid::Uuid::new_v4();
        let course_id = uuid::Uuid::new_v4();
        let subject_id = uuid::Uuid::new_v4();

        let session = session_manager.create_session(
            "Active Session".to_string(),
            folder_id,
            course_id,
            subject_id,
            true,
        );

        let active = session_manager.get_active_session();
        assert!(active.is_some(), "Should return active session");
        assert_eq!(active.unwrap().id, session.id);
    }

    #[tokio::test]
    async fn test_session_persistence_file_io() {
        let sessions_path = get_test_sessions_path();

        // Create and save sessions
        let mut session_manager = SessionManager::new();
        let folder_id = uuid::Uuid::new_v4();
        let course_id = uuid::Uuid::new_v4();
        let subject_id = uuid::Uuid::new_v4();

        session_manager.create_session(
            "Persistent Session".to_string(),
            folder_id,
            course_id,
            subject_id,
            true,
        );

        // Save to file
        session_manager
            .save_to_file(&sessions_path)
            .expect("Failed to save sessions");

        // Load from file
        let loaded_manager =
            SessionManager::load_from_file(&sessions_path).expect("Failed to load sessions");

        // Verify loaded data matches
        assert_eq!(loaded_manager.sessions.len(), 1);
        assert_eq!(
            loaded_manager.sessions[0].name,
            session_manager.sessions[0].name
        );
        assert_eq!(
            loaded_manager.active_session_id,
            session_manager.active_session_id
        );
    }

    #[tokio::test]
    async fn test_session_persistence_nonexistent_file() {
        let sessions_path = std::path::PathBuf::from("/nonexistent/path/sessions.json");

        // Should not crash, should return new empty manager
        let result = SessionManager::load_from_file(&sessions_path);
        assert!(result.is_ok(), "Should handle nonexistent file gracefully");

        let manager = result.unwrap();
        assert_eq!(manager.sessions.len(), 0);
        assert_eq!(manager.active_session_id, None);
    }

    #[tokio::test]
    async fn test_session_persistence_corrupted_file() {
        let dir = tempdir().unwrap();
        let sessions_path = dir.path().join("corrupted.json");

        // Write corrupted JSON
        std::fs::write(&sessions_path, "{ invalid json }").expect("Failed to write file");

        // Should return error for corrupted file
        let result = SessionManager::load_from_file(&sessions_path);
        assert!(result.is_err(), "Should return error for corrupted JSON");
    }

    #[tokio::test]
    async fn test_uuid_parsing_invalid_format() {
        let invalid_uuids = vec![
            "not-a-uuid",
            "12345678",
            "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
            "",
            "00000000-0000-0000-0000-0000000000",  // Too short
            "00000000-0000-0000-0000-0000000000000", // Too long
        ];

        for invalid in invalid_uuids {
            let result = uuid::Uuid::parse_str(invalid);
            assert!(
                result.is_err(),
                "Should reject invalid UUID format: {}",
                invalid
            );
        }
    }

    #[tokio::test]
    async fn test_uuid_parsing_valid_format() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let result = uuid::Uuid::parse_str(valid_uuid);
        assert!(result.is_ok(), "Should accept valid UUID format");
    }

    #[tokio::test]
    async fn test_session_response_from_session_state() {
        let db = setup_test_db().await;

        let user_id = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to create user");

        // Create test entities
        let folder = services::find_or_create_folder(&db, user_id, "Test Folder".to_string())
            .await
            .expect("Failed to create folder");

        let course = services::find_or_create_course(&db, folder.id, "Test Course".to_string())
            .await
            .expect("Failed to create course");

        let subject =
            services::find_or_create_subject(&db, course.id, "Test Subject".to_string())
                .await
                .expect("Failed to create subject");

        // Create session state
        let mut session_manager = SessionManager::new();
        let session = session_manager.create_session(
            "Test Folder / Test Course / Test Subject".to_string(),
            folder.id,
            course.id,
            subject.id,
            false,
        );

        // Convert to response
        let response = SessionResponse::from_session_state(&session, &db)
            .await
            .expect("Failed to create response");

        // Verify response fields
        assert_eq!(response.id, session.id.to_string());
        assert_eq!(response.name, session.name);
        assert_eq!(response.folder_name, "Test Folder");
        assert_eq!(response.course_name, "Test Course");
        assert_eq!(response.subject_name, "Test Subject");
        assert_eq!(response.folder_id, folder.id.to_string());
        assert_eq!(response.course_id, course.id.to_string());
        assert_eq!(response.subject_id, subject.id.to_string());
    }

    #[tokio::test]
    async fn test_session_response_with_deleted_entities() {
        let db = setup_test_db().await;

        let user_id = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to create user");

        let folder = services::find_or_create_folder(&db, user_id, "Test Folder".to_string())
            .await
            .expect("Failed to create folder");

        let course = services::find_or_create_course(&db, folder.id, "Test Course".to_string())
            .await
            .expect("Failed to create course");

        let subject =
            services::find_or_create_subject(&db, course.id, "Test Subject".to_string())
                .await
                .expect("Failed to create subject");

        let mut session_manager = SessionManager::new();
        let session = session_manager.create_session(
            "Test Session".to_string(),
            folder.id,
            course.id,
            subject.id,
            false,
        );

        // Manually delete the subject from DB (simulate deleted entity)
        use crate::db::entities::subjects;
        use sea_orm::EntityTrait;
        subjects::Entity::delete_by_id(subject.id)
            .exec(&db)
            .await
            .expect("Failed to delete subject");

        // Try to create response - should fail
        let result = SessionResponse::from_session_state(&session, &db).await;
        assert!(
            result.is_err(),
            "Should return error when entity not found"
        );
        assert!(result.unwrap_err().contains("not found"));
    }
}
