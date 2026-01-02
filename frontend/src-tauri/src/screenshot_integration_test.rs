// Integration tests for screenshot functionality
// Note: These tests require mocking Tauri's AppHandle which is complex.
// Full integration testing would require a test Tauri app instance.

#[cfg(test)]
mod integration_tests {
    use crate::db::services::screenshots::save_screenshot_to_db;
    use crate::dtos::screenshot::ScreenshotDto;
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
    async fn test_full_screenshot_save_flow() {
        let db = setup_test_db().await;

        // Simulate screenshot DTO
        let dto = ScreenshotDto {
            folder_name: "Computer Science".to_string(),
            course_name: "Data Structures & Algorithms".to_string(),
            subject_name: "Binary Trees".to_string(),
            problem_name: "Lowest Common Ancestor".to_string(),
            base64_data: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string(),
        };

        let image_path = "Computer_Science/Data_Structures_&_Algorithms/Binary_Trees/Lowest_Common_Ancestor.png".to_string();

        // Save to database
        let problem = save_screenshot_to_db(&db, dto, image_path.clone())
            .await
            .expect("Failed to save screenshot");

        // Verify problem was created with correct data
        assert_eq!(problem.title, "Lowest Common Ancestor");
        assert_eq!(problem.image_path, Some(image_path));
        assert_eq!(problem.attempt_count, 0);
        assert_eq!(problem.success_rate, 0.0);
    }

    #[tokio::test]
    async fn test_screenshot_hierarchy_creation_flow() {
        let db = setup_test_db().await;

        // First screenshot
        let dto1 = ScreenshotDto {
            folder_name: "Math".to_string(),
            course_name: "Calculus".to_string(),
            subject_name: "Derivatives".to_string(),
            problem_name: "Chain Rule".to_string(),
            base64_data: "test1".to_string(),
        };

        let path1 = "Math/Calculus/Derivatives/Chain_Rule.png".to_string();
        let problem1 = save_screenshot_to_db(&db, dto1, path1).await.expect("Failed to save");

        // Second screenshot in same hierarchy
        let dto2 = ScreenshotDto {
            folder_name: "Math".to_string(),
            course_name: "Calculus".to_string(),
            subject_name: "Derivatives".to_string(),
            problem_name: "Product Rule".to_string(),
            base64_data: "test2".to_string(),
        };

        let path2 = "Math/Calculus/Derivatives/Product_Rule.png".to_string();
        let problem2 = save_screenshot_to_db(&db, dto2, path2).await.expect("Failed to save");

        // Both problems should be in the same subject
        assert_eq!(problem1.subject_id, problem2.subject_id);

        // Verify hierarchy was reused
        use crate::db::entities::{users, folders, courses, subjects};
        use crate::db::entities::users::Entity as User;
        use crate::db::entities::folders::Entity as Folder;
        use crate::db::entities::courses::Entity as Course;
        use crate::db::entities::subjects::Entity as Subject;

        let user = User::find()
            .filter(users::Column::Email.eq("test@plutodesk.local"))
            .one(&db)
            .await
            .expect("Query failed")
            .expect("User not found");

        let folders = Folder::find()
            .filter(folders::Column::UserId.eq(user.id))
            .all(&db)
            .await
            .expect("Query failed");
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].name, "Math");

        let courses = Course::find()
            .filter(courses::Column::FolderId.eq(folders[0].id))
            .all(&db)
            .await
            .expect("Query failed");
        assert_eq!(courses.len(), 1);
        assert_eq!(courses[0].name, "Calculus");

        let subjects = Subject::find()
            .filter(subjects::Column::CourseId.eq(courses[0].id))
            .all(&db)
            .await
            .expect("Query failed");
        assert_eq!(subjects.len(), 1);
        assert_eq!(subjects[0].name, "Derivatives");
    }

    // Note: Hotkey registration and triggering tests would require:
    // 1. A full Tauri app instance
    // 2. Mocking the global shortcut plugin
    // 3. Simulating keyboard events
    // These are better suited for E2E tests with a test harness
}

