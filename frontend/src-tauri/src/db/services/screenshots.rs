use sea_orm::*;
use crate::db::entities::{
    folders, folders::Entity as Folder,
    courses, courses::Entity as Course,
    sets, sets::Entity as SetEntity,
    problems,
    users, users::Entity as User,
};
use crate::dtos::screenshot::ScreenshotDto;
use uuid::Uuid;

// Default test user email for MVP
const DEFAULT_USER_EMAIL: &str = "test@plutodesk.local";
const DEFAULT_USER_NAME: &str = "Test User";
const DEFAULT_USER_PASSWORD_HASH: &str = "test_hash"; // Placeholder for MVP

/// Get or create a default test user for MVP
pub async fn get_or_create_default_user(
    db: &DatabaseConnection,
) -> Result<Uuid, DbErr> {
    // Try to find existing user by email
    let user = User::find()
        .filter(users::Column::Email.eq(DEFAULT_USER_EMAIL))
        .one(db)
        .await?;

    if let Some(user) = user {
        return Ok(user.id);
    }

    // Create new default user
    let now = chrono::Utc::now().naive_utc();
    let user_id = Uuid::new_v4();

    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(DEFAULT_USER_EMAIL.to_string()),
        password_hash: Set(DEFAULT_USER_PASSWORD_HASH.to_string()),
        name: Set(DEFAULT_USER_NAME.to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        is_premium: Set(false),
        last_sync: Set(None),
    };

    user.insert(db).await?;
    Ok(user_id)
}

/// Find or create a folder by user_id and name
pub async fn find_or_create_folder(
    db: &DatabaseConnection,
    user_id: Uuid,
    name: String,
) -> Result<folders::Model, DbErr> {
    // Try to find existing folder
    let folder = Folder::find()
        .filter(folders::Column::UserId.eq(user_id))
        .filter(folders::Column::Name.eq(&name))
        .one(db)
        .await?;

    if let Some(folder) = folder {
        return Ok(folder);
    }

    // Calculate sort_order (max + 1 or 0 if none exist)
    let max_sort = Folder::find()
        .filter(folders::Column::UserId.eq(user_id))
        .order_by_desc(folders::Column::SortOrder)
        .one(db)
        .await?
        .map(|f| f.sort_order)
        .unwrap_or(-1);

    let sort_order = max_sort + 1;

    // Create new folder
    let now = chrono::Utc::now().naive_utc();
    let folder = folders::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        name: Set(name),
        description: Set(None),
        sort_order: Set(sort_order),
        created_at: Set(now),
        updated_at: Set(now),
        is_synced: Set(false),
    };

    folder.insert(db).await
}

/// Find or create a course by folder_id and name
pub async fn find_or_create_course(
    db: &DatabaseConnection,
    folder_id: Uuid,
    name: String,
) -> Result<courses::Model, DbErr> {
    // Try to find existing course
    let course = Course::find()
        .filter(courses::Column::FolderId.eq(folder_id))
        .filter(courses::Column::Name.eq(&name))
        .one(db)
        .await?;

    if let Some(course) = course {
        return Ok(course);
    }

    // Calculate sort_order (max + 1 or 0 if none exist)
    let max_sort = Course::find()
        .filter(courses::Column::FolderId.eq(folder_id))
        .order_by_desc(courses::Column::SortOrder)
        .one(db)
        .await?
        .map(|c| c.sort_order)
        .unwrap_or(-1);

    let sort_order = max_sort + 1;

    // Create new course
    let now = chrono::Utc::now().naive_utc();
    let course = courses::ActiveModel {
        id: Set(Uuid::new_v4()),
        folder_id: Set(folder_id),
        name: Set(name),
        description: Set(None),
        color_code: Set(None),
        sort_order: Set(sort_order),
        created_at: Set(now),
        updated_at: Set(now),
        is_synced: Set(false),
    };

    course.insert(db).await
}

/// Find or create a set by course_id and name
pub async fn find_or_create_set(
    db: &DatabaseConnection,
    course_id: Uuid,
    name: String,
) -> Result<sets::Model, DbErr> {
    // Try to find existing set
    let set = SetEntity::find()
        .filter(sets::Column::CourseId.eq(course_id))
        .filter(sets::Column::Name.eq(&name))
        .one(db)
        .await?;

    if let Some(set) = set {
        return Ok(set);
    }

    // Calculate sort_order (max + 1 or 0 if none exist)
    let max_sort = SetEntity::find()
        .filter(sets::Column::CourseId.eq(course_id))
        .order_by_desc(sets::Column::SortOrder)
        .one(db)
        .await?
        .map(|s| s.sort_order)
        .unwrap_or(-1);

    let sort_order = max_sort + 1;

    // Create new set
    let now = chrono::Utc::now().naive_utc();
    let set = sets::ActiveModel {
        id: Set(Uuid::new_v4()),
        course_id: Set(course_id),
        name: Set(name),
        description: Set(None),
        sort_order: Set(sort_order),
        created_at: Set(now),
        updated_at: Set(now),
        is_synced: Set(false),
    };

    set.insert(db).await
}

/// Save screenshot data to database, creating all necessary hierarchy entries
pub async fn save_screenshot_to_db(
    db: &DatabaseConnection,
    dto: ScreenshotDto,
    image_path: String,
) -> Result<problems::Model, DbErr> {
    // Get or create default user
    let user_id = get_or_create_default_user(db).await?;

    // Find or create folder
    let folder = find_or_create_folder(db, user_id, dto.folder_name).await?;

    // Find or create course
    let course = find_or_create_course(db, folder.id, dto.course_name).await?;

    // Find or create set
    let set = find_or_create_set(db, course.id, dto.set_name).await?;

    // Create problem with the screenshot
    let now = chrono::Utc::now().naive_utc();
    let problem = problems::ActiveModel {
        id: Set(Uuid::new_v4()),
        set_id: Set(set.id),
        title: Set(dto.problem_name),
        description: Set(None),
        image_path: Set(Some(image_path)),
        s3_image_key: Set(None),
        confidence_level: Set(0),
        notes: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        last_attempted: Set(None),
        attempt_count: Set(0),
        success_rate: Set(0.0),
        is_synced: Set(false),
        last_modified: Set(now),
    };

    problem.insert(db).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use migration::{Migrator, MigratorTrait};
    use sea_orm::Database;
    use crate::dtos::screenshot::ScreenshotDto;

    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");

        db
    }

    #[tokio::test]
    async fn test_get_or_create_default_user_creates_user() {
        let db = setup_test_db().await;

        let user_id = get_or_create_default_user(&db).await.expect("Failed to get/create user");

        // Verify user exists
        let user = User::find_by_id(user_id).one(&db).await.expect("Query failed");
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, DEFAULT_USER_EMAIL);
        assert_eq!(user.name, DEFAULT_USER_NAME);
    }

    #[tokio::test]
    async fn test_get_or_create_default_user_returns_existing() {
        let db = setup_test_db().await;

        let user_id1 = get_or_create_default_user(&db).await.expect("Failed to get/create user");
        let user_id2 = get_or_create_default_user(&db).await.expect("Failed to get/create user");

        // Should return the same user ID
        assert_eq!(user_id1, user_id2);
    }

    #[tokio::test]
    async fn test_find_or_create_folder_creates_folder() {
        let db = setup_test_db().await;
        let user_id = get_or_create_default_user(&db).await.expect("Failed to get/create user");

        let folder = find_or_create_folder(&db, user_id, "Test Folder".to_string())
            .await
            .expect("Failed to find/create folder");

        assert_eq!(folder.name, "Test Folder");
        assert_eq!(folder.user_id, user_id);
        assert_eq!(folder.sort_order, 0);
    }

    #[tokio::test]
    async fn test_find_or_create_folder_returns_existing() {
        let db = setup_test_db().await;
        let user_id = get_or_create_default_user(&db).await.expect("Failed to get/create user");

        let folder1 = find_or_create_folder(&db, user_id, "Test Folder".to_string())
            .await
            .expect("Failed to find/create folder");
        let folder2 = find_or_create_folder(&db, user_id, "Test Folder".to_string())
            .await
            .expect("Failed to find/create folder");

        assert_eq!(folder1.id, folder2.id);
    }

    #[tokio::test]
    async fn test_find_or_create_folder_sort_order_increments() {
        let db = setup_test_db().await;
        let user_id = get_or_create_default_user(&db).await.expect("Failed to get/create user");

        let folder1 = find_or_create_folder(&db, user_id, "Folder 1".to_string())
            .await
            .expect("Failed to find/create folder");
        let folder2 = find_or_create_folder(&db, user_id, "Folder 2".to_string())
            .await
            .expect("Failed to find/create folder");
        let folder3 = find_or_create_folder(&db, user_id, "Folder 3".to_string())
            .await
            .expect("Failed to find/create folder");

        assert_eq!(folder1.sort_order, 0);
        assert_eq!(folder2.sort_order, 1);
        assert_eq!(folder3.sort_order, 2);
    }

    #[tokio::test]
    async fn test_find_or_create_course_creates_course() {
        let db = setup_test_db().await;
        let user_id = get_or_create_default_user(&db).await.expect("Failed to get/create user");
        let folder = find_or_create_folder(&db, user_id, "Test Folder".to_string())
            .await
            .expect("Failed to find/create folder");

        let course = find_or_create_course(&db, folder.id, "Test Course".to_string())
            .await
            .expect("Failed to find/create course");

        assert_eq!(course.name, "Test Course");
        assert_eq!(course.folder_id, folder.id);
        assert_eq!(course.sort_order, 0);
    }

    #[tokio::test]
    async fn test_find_or_create_course_returns_existing() {
        let db = setup_test_db().await;
        let user_id = get_or_create_default_user(&db).await.expect("Failed to get/create user");
        let folder = find_or_create_folder(&db, user_id, "Test Folder".to_string())
            .await
            .expect("Failed to find/create folder");

        let course1 = find_or_create_course(&db, folder.id, "Test Course".to_string())
            .await
            .expect("Failed to find/create course");
        let course2 = find_or_create_course(&db, folder.id, "Test Course".to_string())
            .await
            .expect("Failed to find/create course");

        assert_eq!(course1.id, course2.id);
    }

    #[tokio::test]
    async fn test_find_or_create_course_sort_order_increments() {
        let db = setup_test_db().await;
        let user_id = get_or_create_default_user(&db).await.expect("Failed to get/create user");
        let folder = find_or_create_folder(&db, user_id, "Test Folder".to_string())
            .await
            .expect("Failed to find/create folder");

        let course1 = find_or_create_course(&db, folder.id, "Course 1".to_string())
            .await
            .expect("Failed to find/create course");
        let course2 = find_or_create_course(&db, folder.id, "Course 2".to_string())
            .await
            .expect("Failed to find/create course");

        assert_eq!(course1.sort_order, 0);
        assert_eq!(course2.sort_order, 1);
    }

    #[tokio::test]
    async fn test_find_or_create_set_creates_set() {
        let db = setup_test_db().await;
        let user_id = get_or_create_default_user(&db).await.expect("Failed to get/create user");
        let folder = find_or_create_folder(&db, user_id, "Test Folder".to_string())
            .await
            .expect("Failed to find/create folder");
        let course = find_or_create_course(&db, folder.id, "Test Course".to_string())
            .await
            .expect("Failed to find/create course");

        let set = find_or_create_set(&db, course.id, "Test Set".to_string())
            .await
            .expect("Failed to find/create set");

        assert_eq!(set.name, "Test Set");
        assert_eq!(set.course_id, course.id);
        assert_eq!(set.sort_order, 0);
    }

    #[tokio::test]
    async fn test_find_or_create_set_returns_existing() {
        let db = setup_test_db().await;
        let user_id = get_or_create_default_user(&db).await.expect("Failed to get/create user");
        let folder = find_or_create_folder(&db, user_id, "Test Folder".to_string())
            .await
            .expect("Failed to find/create folder");
        let course = find_or_create_course(&db, folder.id, "Test Course".to_string())
            .await
            .expect("Failed to find/create course");

        let set1 = find_or_create_set(&db, course.id, "Test Set".to_string())
            .await
            .expect("Failed to find/create set");
        let set2 = find_or_create_set(&db, course.id, "Test Set".to_string())
            .await
            .expect("Failed to find/create set");

        assert_eq!(set1.id, set2.id);
    }

    #[tokio::test]
    async fn test_find_or_create_set_sort_order_increments() {
        let db = setup_test_db().await;
        let user_id = get_or_create_default_user(&db).await.expect("Failed to get/create user");
        let folder = find_or_create_folder(&db, user_id, "Test Folder".to_string())
            .await
            .expect("Failed to find/create folder");
        let course = find_or_create_course(&db, folder.id, "Test Course".to_string())
            .await
            .expect("Failed to find/create course");

        let set1 = find_or_create_set(&db, course.id, "Set 1".to_string())
            .await
            .expect("Failed to find/create set");
        let set2 = find_or_create_set(&db, course.id, "Set 2".to_string())
            .await
            .expect("Failed to find/create set");

        assert_eq!(set1.sort_order, 0);
        assert_eq!(set2.sort_order, 1);
    }

    #[tokio::test]
    async fn test_save_screenshot_to_db_creates_full_hierarchy() {
        let db = setup_test_db().await;

        let dto = ScreenshotDto {
            folder_name: "Computer Science".to_string(),
            course_name: "Data Structures & Algorithms".to_string(),
            set_name: "Binary Trees".to_string(),
            problem_name: "Lowest Common Ancestor".to_string(),
            base64_data: "test_base64_data".to_string(),
        };

        let image_path = "Computer_Science/Data_Structures_&_Algorithms/Binary_Trees/Lowest_Common_Ancestor.png".to_string();

        let problem = save_screenshot_to_db(&db, dto, image_path.clone())
            .await
            .expect("Failed to save screenshot");

        // Verify problem was created
        assert_eq!(problem.title, "Lowest Common Ancestor");
        assert_eq!(problem.image_path, Some(image_path.clone()));

        // Verify hierarchy was created
        let user = User::find()
            .filter(users::Column::Email.eq(DEFAULT_USER_EMAIL))
            .one(&db)
            .await
            .expect("Query failed")
            .expect("User not found");

        let folder = Folder::find()
            .filter(folders::Column::UserId.eq(user.id))
            .filter(folders::Column::Name.eq("Computer Science"))
            .one(&db)
            .await
            .expect("Query failed")
            .expect("Folder not found");

        let course = Course::find()
            .filter(courses::Column::FolderId.eq(folder.id))
            .filter(courses::Column::Name.eq("Data Structures & Algorithms"))
            .one(&db)
            .await
            .expect("Query failed")
            .expect("Course not found");

        let set = SetEntity::find()
            .filter(sets::Column::CourseId.eq(course.id))
            .filter(sets::Column::Name.eq("Binary Trees"))
            .one(&db)
            .await
            .expect("Query failed")
            .expect("Set not found");

        assert_eq!(problem.set_id, set.id);
    }

    #[tokio::test]
    async fn test_save_screenshot_to_db_reuses_existing_hierarchy() {
        let db = setup_test_db().await;

        let dto1 = ScreenshotDto {
            folder_name: "Computer Science".to_string(),
            course_name: "Data Structures & Algorithms".to_string(),
            set_name: "Binary Trees".to_string(),
            problem_name: "Problem 1".to_string(),
            base64_data: "test_base64_data_1".to_string(),
        };

        let dto2 = ScreenshotDto {
            folder_name: "Computer Science".to_string(),
            course_name: "Data Structures & Algorithms".to_string(),
            set_name: "Binary Trees".to_string(),
            problem_name: "Problem 2".to_string(),
            base64_data: "test_base64_data_2".to_string(),
        };

        let image_path1 = "Computer_Science/Data_Structures_&_Algorithms/Binary_Trees/Problem_1.png".to_string();
        let image_path2 = "Computer_Science/Data_Structures_&_Algorithms/Binary_Trees/Problem_2.png".to_string();

        let problem1 = save_screenshot_to_db(&db, dto1, image_path1)
            .await
            .expect("Failed to save screenshot");

        let problem2 = save_screenshot_to_db(&db, dto2, image_path2)
            .await
            .expect("Failed to save screenshot");

        // Both problems should be in the same set
        assert_eq!(problem1.set_id, problem2.set_id);

        // Verify only one folder, course, and set exist
        let user = User::find()
            .filter(users::Column::Email.eq(DEFAULT_USER_EMAIL))
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

        let courses = Course::find()
            .filter(courses::Column::FolderId.eq(folders[0].id))
            .all(&db)
            .await
            .expect("Query failed");
        assert_eq!(courses.len(), 1);

        let sets = SetEntity::find()
            .filter(sets::Column::CourseId.eq(courses[0].id))
            .all(&db)
            .await
            .expect("Query failed");
        assert_eq!(sets.len(), 1);
    }
}

