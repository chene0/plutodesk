use sea_orm::*;
use crate::db::entities::{folders, folders::Entity as Folder};
use uuid::Uuid;

pub async fn create_folder(
    db: &DatabaseConnection,
    user_id: Uuid,
    name: String,
    description: Option<String>,
    sort_order: i32,
) -> Result<folders::Model, DbErr> {
    let now = chrono::Utc::now().naive_utc();

    let folder = folders::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        name: Set(name),
        description: Set(description),
        sort_order: Set(sort_order),
        created_at: Set(now),
        updated_at: Set(now),
        is_synced: Set(false),
    };

    folder.insert(db).await
}

pub async fn get_folder_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<folders::Model>, DbErr> {
    Folder::find_by_id(id).one(db).await
}

pub async fn get_folders_by_user(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Vec<folders::Model>, DbErr> {
    Folder::find()
        .filter(folders::Column::UserId.eq(user_id))
        .order_by_asc(folders::Column::SortOrder)
        .all(db)
        .await
}

pub async fn update_folder(
    db: &DatabaseConnection,
    id: Uuid,
    name: Option<String>,
    description: Option<Option<String>>,
    sort_order: Option<i32>,
) -> Result<folders::Model, DbErr> {
    let folder = Folder::find_by_id(id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("Folder not found".to_string()))?;

    let mut folder: folders::ActiveModel = folder.into();

    if let Some(n) = name {
        folder.name = Set(n);
    }
    if let Some(d) = description {
        folder.description = Set(d);
    }
    if let Some(so) = sort_order {
        folder.sort_order = Set(so);
    }

    folder.updated_at = Set(chrono::Utc::now().naive_utc());
    folder.is_synced = Set(false);

    folder.update(db).await
}

pub async fn delete_folder(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<DeleteResult, DbErr> {
    Folder::delete_by_id(id).exec(db).await
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[tokio::test]
    async fn test_create_folder() {
        let db = setup_test_db().await;
        let user_id = create_test_user(&db).await;

        let folder = create_folder(&db, user_id, "Test Folder".to_string(), None, 0)
            .await
            .expect("Failed to create folder");

        assert_eq!(folder.name, "Test Folder");
        assert_eq!(folder.user_id, user_id);
        assert_eq!(folder.sort_order, 0);
        assert!(!folder.is_synced);
    }

    #[tokio::test]
    async fn test_get_folder_by_id() {
        let db = setup_test_db().await;
        let user_id = create_test_user(&db).await;

        let created = create_folder(&db, user_id, "Test Folder".to_string(), None, 0)
            .await
            .expect("Failed to create folder");

        let found = get_folder_by_id(&db, created.id)
            .await
            .expect("Failed to get folder")
            .expect("Folder not found");

        assert_eq!(found.id, created.id);
        assert_eq!(found.name, "Test Folder");
    }

    #[tokio::test]
    async fn test_get_folders_by_user() {
        let db = setup_test_db().await;
        let user_id = create_test_user(&db).await;

        // Create multiple folders
        create_folder(&db, user_id, "Folder A".to_string(), None, 0)
            .await
            .expect("Failed to create folder");
        create_folder(&db, user_id, "Folder B".to_string(), None, 1)
            .await
            .expect("Failed to create folder");
        create_folder(&db, user_id, "Folder C".to_string(), None, 2)
            .await
            .expect("Failed to create folder");

        let folders = get_folders_by_user(&db, user_id)
            .await
            .expect("Failed to get folders");

        assert_eq!(folders.len(), 3);
        assert_eq!(folders[0].name, "Folder A");
        assert_eq!(folders[1].name, "Folder B");
        assert_eq!(folders[2].name, "Folder C");
    }

    #[tokio::test]
    async fn test_update_folder() {
        let db = setup_test_db().await;
        let user_id = create_test_user(&db).await;

        let created = create_folder(&db, user_id, "Original".to_string(), None, 0)
            .await
            .expect("Failed to create folder");

        let updated = update_folder(
            &db,
            created.id,
            Some("Updated".to_string()),
            Some(Some("New description".to_string())),
            Some(5),
        )
        .await
        .expect("Failed to update folder");

        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.description, Some("New description".to_string()));
        assert_eq!(updated.sort_order, 5);
        assert!(!updated.is_synced);
    }

    #[tokio::test]
    async fn test_delete_folder() {
        let db = setup_test_db().await;
        let user_id = create_test_user(&db).await;

        let created = create_folder(&db, user_id, "To Delete".to_string(), None, 0)
            .await
            .expect("Failed to create folder");

        let result = delete_folder(&db, created.id)
            .await
            .expect("Failed to delete folder");

        assert_eq!(result.rows_affected, 1);

        let found = get_folder_by_id(&db, created.id).await.expect("Query failed");
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_folder_isolation_between_users() {
        let db = setup_test_db().await;
        let user1_id = create_test_user(&db).await;
        let user2_id = create_test_user(&db).await;

        create_folder(&db, user1_id, "User 1 Folder".to_string(), None, 0)
            .await
            .expect("Failed to create folder");
        create_folder(&db, user2_id, "User 2 Folder".to_string(), None, 0)
            .await
            .expect("Failed to create folder");

        let user1_folders = get_folders_by_user(&db, user1_id)
            .await
            .expect("Failed to get folders");
        let user2_folders = get_folders_by_user(&db, user2_id)
            .await
            .expect("Failed to get folders");

        assert_eq!(user1_folders.len(), 1);
        assert_eq!(user2_folders.len(), 1);
        assert_eq!(user1_folders[0].name, "User 1 Folder");
        assert_eq!(user2_folders[0].name, "User 2 Folder");
    }
}
