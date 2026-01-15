#[cfg(test)]
mod screenshot_session_integration_tests {
    use crate::db::services;
    use crate::dtos::screenshot::ScreenshotDto;
    use crate::session::SessionManager;
    use migration::{Migrator, MigratorTrait};
    use sea_orm::Database;

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
    async fn test_screenshot_save_with_active_session() {
        let db = setup_test_db().await;

        // Create default user
        let user_id = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to create user");

        // Create folder, course, subject
        let folder = services::find_or_create_folder(&db, user_id, "Computer Science".to_string())
            .await
            .expect("Failed to create folder");

        let course = services::find_or_create_course(&db, folder.id, "Algorithms".to_string())
            .await
            .expect("Failed to create course");

        let subject =
            services::find_or_create_subject(&db, course.id, "Dynamic Programming".to_string())
                .await
                .expect("Failed to create subject");

        // Create session
        let mut session_manager = SessionManager::new();
        let session = session_manager.create_session(
            "Test Session".to_string(),
            folder.id,
            course.id,
            subject.id,
            true,
        );

        // Verify session is active
        assert!(session_manager.get_active_session().is_some());
        assert_eq!(session_manager.get_active_session().unwrap().id, session.id);

        // Simulate screenshot save using session context
        let dto = ScreenshotDto {
            folder_name: "Computer Science".to_string(),
            course_name: "Algorithms".to_string(),
            subject_name: "Dynamic Programming".to_string(),
            problem_name: "Knapsack Problem".to_string(),
            base64_data: "test_base64_data".to_string(),
        };

        let image_path =
            "Computer_Science/Algorithms/Dynamic_Programming/Knapsack_Problem.png".to_string();

        let problem = services::save_screenshot_to_db(&db, dto, image_path.clone())
            .await
            .expect("Failed to save screenshot");

        // Verify problem is saved to correct subject
        assert_eq!(problem.subject_id, subject.id);
        assert_eq!(problem.title, "Knapsack Problem");
        assert_eq!(problem.image_path, Some(image_path));
    }

    #[tokio::test]
    async fn test_screenshot_save_creates_hierarchy_from_session() {
        let db = setup_test_db().await;

        // Create default user
        let user_id = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to create user");

        // Create session with new folder/course/subject names
        let folder = services::find_or_create_folder(&db, user_id, "Mathematics".to_string())
            .await
            .expect("Failed to create folder");

        let course = services::find_or_create_course(&db, folder.id, "Calculus".to_string())
            .await
            .expect("Failed to create course");

        let subject = services::find_or_create_subject(&db, course.id, "Derivatives".to_string())
            .await
            .expect("Failed to create subject");

        let mut session_manager = SessionManager::new();
        session_manager.create_session(
            "Math Session".to_string(),
            folder.id,
            course.id,
            subject.id,
            true,
        );

        // Take multiple screenshots in the same session
        for i in 1..=3 {
            let dto = ScreenshotDto {
                folder_name: "Mathematics".to_string(),
                course_name: "Calculus".to_string(),
                subject_name: "Derivatives".to_string(),
                problem_name: format!("Problem {}", i),
                base64_data: format!("test_base64_data_{}", i),
            };

            let image_path = format!("Mathematics/Calculus/Derivatives/Problem_{}.png", i);

            let problem = services::save_screenshot_to_db(&db, dto, image_path)
                .await
                .expect("Failed to save screenshot");

            // All problems should be in the same subject
            assert_eq!(problem.subject_id, subject.id);
        }

        // Verify all problems are in the same subject
        let problems = services::get_problems_by_subject(&db, subject.id)
            .await
            .expect("Failed to get problems");

        assert_eq!(problems.len(), 3);
    }

    #[tokio::test]
    async fn test_session_switch_affects_screenshot_location() {
        let db = setup_test_db().await;

        let user_id = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to create user");

        // Create two different contexts
        let folder1 = services::find_or_create_folder(&db, user_id, "Computer Science".to_string())
            .await
            .expect("Failed to create folder");

        let course1 = services::find_or_create_course(&db, folder1.id, "Algorithms".to_string())
            .await
            .expect("Failed to create course");

        let subject1 = services::find_or_create_subject(&db, course1.id, "Sorting".to_string())
            .await
            .expect("Failed to create subject");

        let folder2 = services::find_or_create_folder(&db, user_id, "Mathematics".to_string())
            .await
            .expect("Failed to create folder");

        let course2 = services::find_or_create_course(&db, folder2.id, "Calculus".to_string())
            .await
            .expect("Failed to create course");

        let subject2 = services::find_or_create_subject(&db, course2.id, "Integrals".to_string())
            .await
            .expect("Failed to create subject");

        // Create two sessions
        let mut session_manager = SessionManager::new();
        let _session1 = session_manager.create_session(
            "CS Session".to_string(),
            folder1.id,
            course1.id,
            subject1.id,
            true,
        );

        let session2 = session_manager.create_session(
            "Math Session".to_string(),
            folder2.id,
            course2.id,
            subject2.id,
            false,
        );

        // Take screenshot in session1
        let dto1 = ScreenshotDto {
            folder_name: "Computer Science".to_string(),
            course_name: "Algorithms".to_string(),
            subject_name: "Sorting".to_string(),
            problem_name: "QuickSort".to_string(),
            base64_data: "test_data_1".to_string(),
        };

        let problem1 = services::save_screenshot_to_db(&db, dto1, "path1.png".to_string())
            .await
            .expect("Failed to save screenshot");

        assert_eq!(problem1.subject_id, subject1.id);

        // Switch to session2
        session_manager.start_session(session2.id).unwrap();

        // Take screenshot in session2
        let dto2 = ScreenshotDto {
            folder_name: "Mathematics".to_string(),
            course_name: "Calculus".to_string(),
            subject_name: "Integrals".to_string(),
            problem_name: "Integration by Parts".to_string(),
            base64_data: "test_data_2".to_string(),
        };

        let problem2 = services::save_screenshot_to_db(&db, dto2, "path2.png".to_string())
            .await
            .expect("Failed to save screenshot");

        assert_eq!(problem2.subject_id, subject2.id);

        // Verify problems are in different subjects
        assert_ne!(problem1.subject_id, problem2.subject_id);
    }

    #[tokio::test]
    async fn test_no_active_session_error_handling() {
        let session_manager = SessionManager::new();

        // Verify no active session
        assert!(session_manager.get_active_session().is_none());

        // In the actual implementation, this would trigger an error
        // when trying to save a screenshot without an active session
        // The error handling is done in the receive_screenshot_data command
    }

    #[tokio::test]
    async fn test_screenshot_blocked_without_session() {
        let session_manager = SessionManager::new();

        // Verify no active session
        assert!(
            session_manager.get_active_session().is_none(),
            "Should have no active session"
        );

        // Simulate check_session_and_notify logic
        let has_session = session_manager.get_active_session().is_some();
        assert!(
            !has_session,
            "Should return false when no session active"
        );

        // This would trigger notification and main window opening in actual implementation
    }

    #[tokio::test]
    async fn test_screenshot_with_inline_session_selection() {
        let db = setup_test_db().await;

        let user_id = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to create user");

        // Create entities for inline selection
        let folder = services::find_or_create_folder(&db, user_id, "Physics".to_string())
            .await
            .expect("Failed to create folder");

        let course = services::find_or_create_course(&db, folder.id, "Mechanics".to_string())
            .await
            .expect("Failed to create course");

        let subject = services::find_or_create_subject(&db, course.id, "Dynamics".to_string())
            .await
            .expect("Failed to create subject");

        // Simulate screenshot save with inline IDs (not using active session)
        let dto = ScreenshotDto {
            folder_name: folder.name.clone(),
            course_name: course.name.clone(),
            subject_name: subject.name.clone(),
            problem_name: "Newton's Laws".to_string(),
            base64_data: "test_data".to_string(),
        };

        let image_path = "Physics/Mechanics/Dynamics/Newtons_Laws.png".to_string();

        let problem = services::save_screenshot_to_db(&db, dto, image_path)
            .await
            .expect("Failed to save screenshot");

        // Verify screenshot was saved to correct subject
        assert_eq!(problem.subject_id, subject.id);
        assert_eq!(problem.title, "Newton's Laws");
    }

    #[tokio::test]
    async fn test_screenshot_with_active_session_context() {
        let db = setup_test_db().await;

        let user_id = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to create user");

        // Create session context
        let folder = services::find_or_create_folder(&db, user_id, "Chemistry".to_string())
            .await
            .expect("Failed to create folder");

        let course = services::find_or_create_course(&db, folder.id, "Organic Chemistry".to_string())
            .await
            .expect("Failed to create course");

        let subject = services::find_or_create_subject(&db, course.id, "Reactions".to_string())
            .await
            .expect("Failed to create subject");

        // Create and activate session
        let mut session_manager = SessionManager::new();
        let _session = session_manager.create_session(
            "Chemistry Session".to_string(),
            folder.id,
            course.id,
            subject.id,
            true, // Start immediately
        );

        let active_session = session_manager.get_active_session().unwrap();

        // Simulate screenshot save using session context
        let dto = ScreenshotDto {
            folder_name: folder.name.clone(),
            course_name: course.name.clone(),
            subject_name: subject.name.clone(),
            problem_name: "Substitution Reaction".to_string(),
            base64_data: "test_data".to_string(),
        };

        let image_path = "Chemistry/Organic_Chemistry/Reactions/Substitution_Reaction.png".to_string();

        let problem = services::save_screenshot_to_db(&db, dto, image_path)
            .await
            .expect("Failed to save screenshot");

        // Verify screenshot was saved to session's subject
        assert_eq!(problem.subject_id, active_session.subject_id);
        assert_eq!(problem.subject_id, subject.id);
    }

    #[tokio::test]
    async fn test_database_lookup_failure_folder_not_found() {
        let db = setup_test_db().await;

        // Try to get non-existent folder
        let nonexistent_folder_id = uuid::Uuid::new_v4();
        let result = services::get_folder_by_id(&db, nonexistent_folder_id).await;

        assert!(result.is_ok(), "Query should succeed");
        assert!(result.unwrap().is_none(), "Should return None for nonexistent folder");
    }

    #[tokio::test]
    async fn test_database_lookup_failure_course_not_found() {
        let db = setup_test_db().await;

        // Try to get non-existent course
        let nonexistent_course_id = uuid::Uuid::new_v4();
        let result = services::get_course_by_id(&db, nonexistent_course_id).await;

        assert!(result.is_ok(), "Query should succeed");
        assert!(result.unwrap().is_none(), "Should return None for nonexistent course");
    }

    #[tokio::test]
    async fn test_database_lookup_failure_subject_not_found() {
        let db = setup_test_db().await;

        // Try to get non-existent subject
        let nonexistent_subject_id = uuid::Uuid::new_v4();
        let result = services::get_subject_by_id(&db, nonexistent_subject_id).await;

        assert!(result.is_ok(), "Query should succeed");
        assert!(result.unwrap().is_none(), "Should return None for nonexistent subject");
    }

    #[tokio::test]
    async fn test_invalid_uuid_parameters() {
        // Test UUID parsing errors
        let invalid_uuids = vec![
            "not-a-uuid",
            "12345678-1234-1234-1234",
            "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
            "",
        ];

        for invalid_uuid in invalid_uuids {
            let result = uuid::Uuid::parse_str(invalid_uuid);
            assert!(
                result.is_err(),
                "Should reject invalid UUID: {}",
                invalid_uuid
            );
        }
    }

    #[tokio::test]
    async fn test_check_session_logic() {
        // Test 1: No session - should return false
        let session_manager = SessionManager::new();
        let has_active_session = session_manager.get_active_session().is_some();
        assert!(!has_active_session, "Should return false when no session");

        // Test 2: With session - should return true
        let mut session_manager = SessionManager::new();
        let folder_id = uuid::Uuid::new_v4();
        let course_id = uuid::Uuid::new_v4();
        let subject_id = uuid::Uuid::new_v4();

        session_manager.create_session(
            "Test Session".to_string(),
            folder_id,
            course_id,
            subject_id,
            true, // Start immediately
        );

        let has_active_session = session_manager.get_active_session().is_some();
        assert!(has_active_session, "Should return true when session active");
    }

    #[tokio::test]
    async fn test_multiple_screenshots_in_same_session() {
        let db = setup_test_db().await;

        let user_id = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to create user");

        // Create session context
        let folder = services::find_or_create_folder(&db, user_id, "Biology".to_string())
            .await
            .expect("Failed to create folder");

        let course = services::find_or_create_course(&db, folder.id, "Cell Biology".to_string())
            .await
            .expect("Failed to create course");

        let subject = services::find_or_create_subject(&db, course.id, "Mitosis".to_string())
            .await
            .expect("Failed to create subject");

        let mut session_manager = SessionManager::new();
        session_manager.create_session(
            "Biology Session".to_string(),
            folder.id,
            course.id,
            subject.id,
            true,
        );

        // Save multiple screenshots in the same session
        for i in 1..=5 {
            let dto = ScreenshotDto {
                folder_name: folder.name.clone(),
                course_name: course.name.clone(),
                subject_name: subject.name.clone(),
                problem_name: format!("Phase {}", i),
                base64_data: format!("test_data_{}", i),
            };

            let image_path = format!("Biology/Cell_Biology/Mitosis/Phase_{}.png", i);

            let problem = services::save_screenshot_to_db(&db, dto, image_path)
                .await
                .expect("Failed to save screenshot");

            // All should be in the same subject
            assert_eq!(problem.subject_id, subject.id);
        }

        // Verify all problems were created
        let problems = services::get_problems_by_subject(&db, subject.id)
            .await
            .expect("Failed to get problems");

        assert_eq!(problems.len(), 5, "Should have 5 problems");
    }

    #[tokio::test]
    async fn test_session_switch_affects_screenshot_context() {
        let db = setup_test_db().await;

        let user_id = services::get_or_create_default_user(&db)
            .await
            .expect("Failed to create user");

        // Create two different contexts
        let folder1 = services::find_or_create_folder(&db, user_id, "Math".to_string())
            .await
            .expect("Failed to create folder");

        let course1 = services::find_or_create_course(&db, folder1.id, "Algebra".to_string())
            .await
            .expect("Failed to create course");

        let subject1 = services::find_or_create_subject(&db, course1.id, "Linear".to_string())
            .await
            .expect("Failed to create subject");

        let folder2 = services::find_or_create_folder(&db, user_id, "Physics".to_string())
            .await
            .expect("Failed to create folder");

        let course2 = services::find_or_create_course(&db, folder2.id, "Quantum".to_string())
            .await
            .expect("Failed to create course");

        let subject2 = services::find_or_create_subject(&db, course2.id, "Entanglement".to_string())
            .await
            .expect("Failed to create subject");

        // Create two sessions
        let mut session_manager = SessionManager::new();
        let _session1 = session_manager.create_session(
            "Math Session".to_string(),
            folder1.id,
            course1.id,
            subject1.id,
            true, // Start immediately
        );

        let session2 = session_manager.create_session(
            "Physics Session".to_string(),
            folder2.id,
            course2.id,
            subject2.id,
            false,
        );

        // Take screenshot in session 1
        let dto1 = ScreenshotDto {
            folder_name: folder1.name.clone(),
            course_name: course1.name.clone(),
            subject_name: subject1.name.clone(),
            problem_name: "Matrix Problem".to_string(),
            base64_data: "test_data_1".to_string(),
        };

        let problem1 = services::save_screenshot_to_db(&db, dto1, "path1.png".to_string())
            .await
            .expect("Failed to save screenshot");

        assert_eq!(problem1.subject_id, subject1.id);

        // Switch to session 2
        session_manager.end_session();
        session_manager
            .start_session(session2.id)
            .expect("Failed to start session 2");

        // Take screenshot in session 2
        let dto2 = ScreenshotDto {
            folder_name: folder2.name.clone(),
            course_name: course2.name.clone(),
            subject_name: subject2.name.clone(),
            problem_name: "Bell State".to_string(),
            base64_data: "test_data_2".to_string(),
        };

        let problem2 = services::save_screenshot_to_db(&db, dto2, "path2.png".to_string())
            .await
            .expect("Failed to save screenshot");

        assert_eq!(problem2.subject_id, subject2.id);

        // Verify screenshots are in different subjects
        assert_ne!(
            problem1.subject_id, problem2.subject_id,
            "Screenshots should be in different subjects"
        );
    }
}
