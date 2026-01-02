use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

use super::Db;

pub async fn init_sqlite(app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Get the app data directory
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    // Create the database path
    let db_path = app_data_dir.join("plutodesk.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    log::info!("Connecting to database at {}", db_url);

    let conn = Database::connect(&db_url).await?;

    // Run migrations
    log::info!("Running database migrations...");
    Migrator::up(&conn, None).await?;
    log::info!("Migrations completed successfully");

    app_handle.manage(Db(Arc::new(conn)));

    log::info!("Database initialized successfully");
    Ok(())
}
