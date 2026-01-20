use sea_orm::*;
use crate::db::entities::{problems, problems::Entity as Problem};
use uuid::Uuid;

pub async fn create_problem(
    db: &DatabaseConnection,
    set_id: Uuid,
    title: String,
    description: Option<String>,
    image_path: Option<String>,
    s3_image_key: Option<String>,
) -> Result<problems::Model, DbErr> {
    let now = chrono::Utc::now().naive_utc();

    let problem = problems::ActiveModel {
        id: Set(Uuid::new_v4()),
        set_id: Set(set_id),
        title: Set(title),
        description: Set(description),
        image_path: Set(image_path),
        s3_image_key: Set(s3_image_key),
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

pub async fn get_problem_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<problems::Model>, DbErr> {
    Problem::find_by_id(id).one(db).await
}

pub async fn get_problems_by_set(
    db: &DatabaseConnection,
    set_id: Uuid,
) -> Result<Vec<problems::Model>, DbErr> {
    Problem::find()
        .filter(problems::Column::SetId.eq(set_id))
        .order_by_desc(problems::Column::CreatedAt)
        .all(db)
        .await
}

pub async fn update_problem(
    db: &DatabaseConnection,
    id: Uuid,
    title: Option<String>,
    description: Option<Option<String>>,
    image_path: Option<Option<String>>,
    s3_image_key: Option<Option<String>>,
    confidence_level: Option<i32>,
    notes: Option<Option<String>>,
) -> Result<problems::Model, DbErr> {
    let problem = Problem::find_by_id(id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("Problem not found".to_string()))?;

    let mut problem: problems::ActiveModel = problem.into();

    if let Some(t) = title {
        problem.title = Set(t);
    }
    if let Some(d) = description {
        problem.description = Set(d);
    }
    if let Some(ip) = image_path {
        problem.image_path = Set(ip);
    }
    if let Some(s3) = s3_image_key {
        problem.s3_image_key = Set(s3);
    }
    if let Some(cl) = confidence_level {
        problem.confidence_level = Set(cl);
    }
    if let Some(n) = notes {
        problem.notes = Set(n);
    }

    let now = chrono::Utc::now().naive_utc();
    problem.updated_at = Set(now);
    problem.last_modified = Set(now);
    problem.is_synced = Set(false);

    problem.update(db).await
}

pub async fn update_problem_stats(
    db: &DatabaseConnection,
    id: Uuid,
    was_successful: bool,
) -> Result<problems::Model, DbErr> {
    let problem = Problem::find_by_id(id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("Problem not found".to_string()))?;

    let new_attempt_count = problem.attempt_count + 1;
    let new_success_rate = if was_successful {
        (problem.success_rate * problem.attempt_count as f32 + 1.0) / new_attempt_count as f32
    } else {
        (problem.success_rate * problem.attempt_count as f32) / new_attempt_count as f32
    };

    let mut problem: problems::ActiveModel = problem.into();
    let now = chrono::Utc::now().naive_utc();

    problem.last_attempted = Set(Some(now));
    problem.attempt_count = Set(new_attempt_count);
    problem.success_rate = Set(new_success_rate);
    problem.updated_at = Set(now);
    problem.last_modified = Set(now);
    problem.is_synced = Set(false);

    problem.update(db).await
}

pub async fn delete_problem(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<DeleteResult, DbErr> {
    Problem::delete_by_id(id).exec(db).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::services::folders::create_folder;
    use crate::db::services::courses::create_course;
    use crate::db::services::sets::create_set;
    use migration::{Migrator, MigratorTrait};
    use sea_orm::Database;

    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");

        db
    }

    async fn create_test_user(db: &DatabaseConnection) -> Uuid {
        use crate::db::entities::users;
        let user_id = Uuid::new_v4();
        let now = chrono::Utc::now().naive_utc();

        let user = users::ActiveModel {
            id: Set(user_id),
            email: Set(format!("test-{}@example.com", user_id)),
            password_hash: Set("hashed_password".to_string()),
            name: Set("Test User".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            is_premium: Set(false),
            last_sync: Set(None),
        };

        user.insert(db).await.expect("Failed to create test user");
        user_id
    }

    async fn create_test_set(db: &DatabaseConnection) -> Uuid {
        let user_id = create_test_user(db).await;
        let folder = create_folder(db, user_id, "Test Folder".to_string(), None, 0)
            .await
            .expect("Failed to create folder");

        let course = create_course(
            db,
            folder.id,
            "Test Course".to_string(),
            None,
            None,
            0,
        )
        .await
        .expect("Failed to create course");

        let set = create_set(
            db,
            course.id,
            "Test Set".to_string(),
            None,
            0,
        )
        .await
        .expect("Failed to create set");

        set.id
    }

    #[tokio::test]
    async fn test_create_problem() {
        let db = setup_test_db().await;
        let set_id = create_test_set(&db).await;

        let problem = create_problem(
            &db,
            set_id,
            "Test Problem".to_string(),
            Some("Description".to_string()),
            None,
            None,
        )
        .await
        .expect("Failed to create problem");

        assert_eq!(problem.title, "Test Problem");
        assert_eq!(problem.description, Some("Description".to_string()));
        assert_eq!(problem.confidence_level, 0);
        assert_eq!(problem.attempt_count, 0);
        assert_eq!(problem.success_rate, 0.0);
    }

    #[tokio::test]
    async fn test_update_problem_stats_success() {
        let db = setup_test_db().await;
        let set_id = create_test_set(&db).await;

        let problem = create_problem(
            &db,
            set_id,
            "Test Problem".to_string(),
            None,
            None,
            None,
        )
        .await
        .expect("Failed to create problem");

        // First successful attempt
        let updated = update_problem_stats(&db, problem.id, true)
            .await
            .expect("Failed to update stats");

        assert_eq!(updated.attempt_count, 1);
        assert_eq!(updated.success_rate, 1.0);
        assert!(updated.last_attempted.is_some());
    }

    #[tokio::test]
    async fn test_update_problem_stats_failure() {
        let db = setup_test_db().await;
        let set_id = create_test_set(&db).await;

        let problem = create_problem(
            &db,
            set_id,
            "Test Problem".to_string(),
            None,
            None,
            None,
        )
        .await
        .expect("Failed to create problem");

        // Failed attempt
        let updated = update_problem_stats(&db, problem.id, false)
            .await
            .expect("Failed to update stats");

        assert_eq!(updated.attempt_count, 1);
        assert_eq!(updated.success_rate, 0.0);
    }

    #[tokio::test]
    async fn test_update_problem_stats_mixed() {
        let db = setup_test_db().await;
        let set_id = create_test_set(&db).await;

        let mut problem = create_problem(
            &db,
            set_id,
            "Test Problem".to_string(),
            None,
            None,
            None,
        )
        .await
        .expect("Failed to create problem");

        // 3 successful attempts
        problem = update_problem_stats(&db, problem.id, true)
            .await
            .expect("Failed to update stats");
        problem = update_problem_stats(&db, problem.id, true)
            .await
            .expect("Failed to update stats");
        problem = update_problem_stats(&db, problem.id, true)
            .await
            .expect("Failed to update stats");

        assert_eq!(problem.attempt_count, 3);
        assert_eq!(problem.success_rate, 1.0);

        // 1 failed attempt
        problem = update_problem_stats(&db, problem.id, false)
            .await
            .expect("Failed to update stats");

        assert_eq!(problem.attempt_count, 4);
        assert_eq!(problem.success_rate, 0.75); // 3/4
    }

    #[tokio::test]
    async fn test_get_problems_by_set() {
        let db = setup_test_db().await;
        let set_id = create_test_set(&db).await;

        // Create multiple problems
        create_problem(&db, set_id, "Problem 1".to_string(), None, None, None)
            .await
            .expect("Failed to create problem");
        create_problem(&db, set_id, "Problem 2".to_string(), None, None, None)
            .await
            .expect("Failed to create problem");
        create_problem(&db, set_id, "Problem 3".to_string(), None, None, None)
            .await
            .expect("Failed to create problem");

        let problems = get_problems_by_set(&db, set_id)
            .await
            .expect("Failed to get problems");

        assert_eq!(problems.len(), 3);
    }

    #[tokio::test]
    async fn test_update_problem() {
        let db = setup_test_db().await;
        let set_id = create_test_set(&db).await;

        let problem = create_problem(
            &db,
            set_id,
            "Original".to_string(),
            None,
            None,
            None,
        )
        .await
        .expect("Failed to create problem");

        let updated = update_problem(
            &db,
            problem.id,
            Some("Updated Title".to_string()),
            Some(Some("New description".to_string())),
            None,
            None,
            Some(5),
            Some(Some("Some notes".to_string())),
        )
        .await
        .expect("Failed to update problem");

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.description, Some("New description".to_string()));
        assert_eq!(updated.confidence_level, 5);
        assert_eq!(updated.notes, Some("Some notes".to_string()));
    }

    #[tokio::test]
    async fn test_delete_problem() {
        let db = setup_test_db().await;
        let set_id = create_test_set(&db).await;

        let problem = create_problem(
            &db,
            set_id,
            "To Delete".to_string(),
            None,
            None,
            None,
        )
        .await
        .expect("Failed to create problem");

        let result = delete_problem(&db, problem.id)
            .await
            .expect("Failed to delete problem");

        assert_eq!(result.rows_affected, 1);

        let found = get_problem_by_id(&db, problem.id)
            .await
            .expect("Query failed");
        assert!(found.is_none());
    }
}
