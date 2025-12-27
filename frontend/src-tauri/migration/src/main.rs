use sea_orm_migration::prelude::*;

#[async_std::main]
async fn main() {
    // Initialize logger
    env_logger::init();

    // Load .env file if it exists (for development)
    if let Err(e) = dotenvy::dotenv() {
        log::debug!("No .env file found or error loading it: {}", e);
    }

    // Get command line arguments
    let args: Vec<String> = std::env::args().collect();

    // Check if the user wants to seed the database
    if args.len() > 1 && args[1] == "seed" {
        // Get database URL from environment or use default
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite::memory:".to_string());

        println!("Connecting to database: {}", db_url);

        let db = sea_orm::Database::connect(&db_url)
            .await
            .expect("Failed to connect to database");

        // Run migrations first
        println!("Running migrations...");
        migration::Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");

        // Seed the database
        println!("Seeding database...");
        migration::seed::seed(&db)
            .await
            .expect("Failed to seed database");

        println!("Seeding completed successfully!");
    } else {
        // Run normal migration CLI
        cli::run_cli(migration::Migrator).await;
    }
}
