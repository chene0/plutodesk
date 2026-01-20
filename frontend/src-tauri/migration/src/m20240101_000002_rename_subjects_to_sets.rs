use sea_orm_migration::prelude::*;
use sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite doesn't support ALTER TABLE RENAME COLUMN directly,
        // so we need to use raw SQL for table and column renames
        
        // Step 1: Rename the subjects table to sets
        let db = manager.get_connection();
        let stmt = Statement::from_sql_and_values(
            manager.get_database_backend(),
            "ALTER TABLE subjects RENAME TO sets",
            [],
        );
        db.execute(stmt).await?;

        // Step 2: Rename subject_id column to set_id in problems table
        // For SQLite, we need to recreate the table with the new column name
        // Step 2a: Create a new problems table with set_id
        let stmt = Statement::from_sql_and_values(
            manager.get_database_backend(),
            "CREATE TABLE problems_new (
                id TEXT PRIMARY KEY NOT NULL,
                set_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                image_path TEXT,
                s3_image_key TEXT,
                confidence_level INTEGER NOT NULL DEFAULT 0,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_attempted TEXT,
                attempt_count INTEGER NOT NULL DEFAULT 0,
                success_rate REAL NOT NULL DEFAULT 0.0,
                is_synced INTEGER NOT NULL DEFAULT 0,
                last_modified TEXT NOT NULL,
                FOREIGN KEY (set_id) REFERENCES sets(id) ON DELETE CASCADE
            )",
            [],
        );
        db.execute(stmt).await?;

        // Step 2b: Copy data from old problems table to new one
        let stmt = Statement::from_sql_and_values(
            manager.get_database_backend(),
            "INSERT INTO problems_new (
                id, set_id, title, description, image_path, s3_image_key,
                confidence_level, notes, created_at, updated_at, last_attempted,
                attempt_count, success_rate, is_synced, last_modified
            )
            SELECT 
                id, subject_id, title, description, image_path, s3_image_key,
                confidence_level, notes, created_at, updated_at, last_attempted,
                attempt_count, success_rate, is_synced, last_modified
            FROM problems",
            [],
        );
        db.execute(stmt).await?;

        // Step 2c: Drop the old problems table
        let stmt = Statement::from_sql_and_values(
            manager.get_database_backend(),
            "DROP TABLE problems",
            [],
        );
        db.execute(stmt).await?;

        // Step 2d: Rename the new table to problems
        let stmt = Statement::from_sql_and_values(
            manager.get_database_backend(),
            "ALTER TABLE problems_new RENAME TO problems",
            [],
        );
        db.execute(stmt).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Reverse the migration: rename sets back to subjects and set_id back to subject_id
        
        // Step 1: Rename sets table back to subjects
        let db = manager.get_connection();
        let stmt = Statement::from_sql_and_values(
            manager.get_database_backend(),
            "ALTER TABLE sets RENAME TO subjects",
            [],
        );
        db.execute(stmt).await?;

        // Step 2: Recreate problems table with subject_id
        // Step 2a: Create a new problems table with subject_id
        let stmt = Statement::from_sql_and_values(
            manager.get_database_backend(),
            "CREATE TABLE problems_new (
                id TEXT PRIMARY KEY NOT NULL,
                subject_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                image_path TEXT,
                s3_image_key TEXT,
                confidence_level INTEGER NOT NULL DEFAULT 0,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_attempted TEXT,
                attempt_count INTEGER NOT NULL DEFAULT 0,
                success_rate REAL NOT NULL DEFAULT 0.0,
                is_synced INTEGER NOT NULL DEFAULT 0,
                last_modified TEXT NOT NULL,
                FOREIGN KEY (subject_id) REFERENCES subjects(id) ON DELETE CASCADE
            )",
            [],
        );
        db.execute(stmt).await?;

        // Step 2b: Copy data from problems table to new one
        let stmt = Statement::from_sql_and_values(
            manager.get_database_backend(),
            "INSERT INTO problems_new (
                id, subject_id, title, description, image_path, s3_image_key,
                confidence_level, notes, created_at, updated_at, last_attempted,
                attempt_count, success_rate, is_synced, last_modified
            )
            SELECT 
                id, set_id, title, description, image_path, s3_image_key,
                confidence_level, notes, created_at, updated_at, last_attempted,
                attempt_count, success_rate, is_synced, last_modified
            FROM problems",
            [],
        );
        db.execute(stmt).await?;

        // Step 2c: Drop the old problems table
        let stmt = Statement::from_sql_and_values(
            manager.get_database_backend(),
            "DROP TABLE problems",
            [],
        );
        db.execute(stmt).await?;

        // Step 2d: Rename the new table to problems
        let stmt = Statement::from_sql_and_values(
            manager.get_database_backend(),
            "ALTER TABLE problems_new RENAME TO problems",
            [],
        );
        db.execute(stmt).await?;

        Ok(())
    }
}

