mod init;
pub mod entities;
pub mod services;

use sea_orm::DatabaseConnection;
use std::sync::Arc;

// Re-export initialization function
pub use init::init_sqlite;

// Main DB struct for app state
pub struct Db(pub Arc<DatabaseConnection>);

// Helper methods that can be used across all models
impl Db {
    pub fn connection(&self) -> &DatabaseConnection {
        &self.0
    }
}
