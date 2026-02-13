pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20240101_000002_rename_subjects_to_sets;
mod m20240213_000003_add_difficulty_to_problems;
pub mod seed;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20240101_000002_rename_subjects_to_sets::Migration),
            Box::new(m20240213_000003_add_difficulty_to_problems::Migration),
        ]
    }
}
