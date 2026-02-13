use sea_orm_migration::prelude::*;
use sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Add the column
        manager
            .alter_table(
                Table::alter()
                    .table(Problems::Table)
                    .add_column(
                        ColumnDef::new(Problems::DifficultyRating)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        // Backfill from most recent attempt per problem
        let stmt = Statement::from_sql_and_values(
            manager.get_database_backend(),
            r#"
            UPDATE problems SET difficulty_rating = (
                SELECT difficulty_rating FROM problem_attempts
                WHERE problem_attempts.problem_id = problems.id
                ORDER BY attempted_at DESC LIMIT 1
            )
            WHERE EXISTS (
                SELECT 1 FROM problem_attempts WHERE problem_attempts.problem_id = problems.id
            )
            "#,
            [],
        );
        db.execute(stmt).await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite doesn't support DROP COLUMN directly - would need table recreation.
        // For simplicity, we leave the column in place on rollback (common practice for SQLite).
        // If down migration is critical, a full table recreate could be added.
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Problems {
    Table,
    DifficultyRating,
}
