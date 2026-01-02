use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(uuid(Users::Id).primary_key())
                    .col(string(Users::Email).unique_key())
                    .col(string(Users::PasswordHash))
                    .col(string(Users::Name))
                    .col(timestamp(Users::CreatedAt))
                    .col(timestamp(Users::UpdatedAt))
                    .col(boolean(Users::IsPremium).default(false))
                    .col(timestamp_null(Users::LastSync))
                    .to_owned(),
            )
            .await?;

        // Create subscriptions table
        manager
            .create_table(
                Table::create()
                    .table(Subscriptions::Table)
                    .if_not_exists()
                    .col(uuid(Subscriptions::Id).primary_key())
                    .col(uuid(Subscriptions::UserId))
                    .col(string(Subscriptions::StripeCustomerId))
                    .col(string(Subscriptions::StripeSubscriptionId))
                    .col(string(Subscriptions::Status))
                    .col(timestamp(Subscriptions::CurrentPeriodStart))
                    .col(timestamp(Subscriptions::CurrentPeriodEnd))
                    .col(timestamp(Subscriptions::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Subscriptions::Table, Subscriptions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create folders table
        manager
            .create_table(
                Table::create()
                    .table(Folders::Table)
                    .if_not_exists()
                    .col(uuid(Folders::Id).primary_key())
                    .col(uuid(Folders::UserId))
                    .col(string(Folders::Name))
                    .col(string_null(Folders::Description))
                    .col(integer(Folders::SortOrder).default(0))
                    .col(timestamp(Folders::CreatedAt))
                    .col(timestamp(Folders::UpdatedAt))
                    .col(boolean(Folders::IsSynced).default(false))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Folders::Table, Folders::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create courses table
        manager
            .create_table(
                Table::create()
                    .table(Courses::Table)
                    .if_not_exists()
                    .col(uuid(Courses::Id).primary_key())
                    .col(uuid(Courses::FolderId))
                    .col(string(Courses::Name))
                    .col(string_null(Courses::Description))
                    .col(string_null(Courses::ColorCode))
                    .col(integer(Courses::SortOrder).default(0))
                    .col(timestamp(Courses::CreatedAt))
                    .col(timestamp(Courses::UpdatedAt))
                    .col(boolean(Courses::IsSynced).default(false))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Courses::Table, Courses::FolderId)
                            .to(Folders::Table, Folders::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create subjects table
        manager
            .create_table(
                Table::create()
                    .table(Subjects::Table)
                    .if_not_exists()
                    .col(uuid(Subjects::Id).primary_key())
                    .col(uuid(Subjects::CourseId))
                    .col(string(Subjects::Name))
                    .col(string_null(Subjects::Description))
                    .col(integer(Subjects::SortOrder).default(0))
                    .col(timestamp(Subjects::CreatedAt))
                    .col(timestamp(Subjects::UpdatedAt))
                    .col(boolean(Subjects::IsSynced).default(false))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Subjects::Table, Subjects::CourseId)
                            .to(Courses::Table, Courses::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create problems table
        manager
            .create_table(
                Table::create()
                    .table(Problems::Table)
                    .if_not_exists()
                    .col(uuid(Problems::Id).primary_key())
                    .col(uuid(Problems::SubjectId))
                    .col(string(Problems::Title))
                    .col(string_null(Problems::Description))
                    .col(string_null(Problems::ImagePath))
                    .col(string_null(Problems::S3ImageKey))
                    .col(integer(Problems::ConfidenceLevel).default(0))
                    .col(string_null(Problems::Notes))
                    .col(timestamp(Problems::CreatedAt))
                    .col(timestamp(Problems::UpdatedAt))
                    .col(timestamp_null(Problems::LastAttempted))
                    .col(integer(Problems::AttemptCount).default(0))
                    .col(float(Problems::SuccessRate).default(0.0))
                    .col(boolean(Problems::IsSynced).default(false))
                    .col(timestamp(Problems::LastModified))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Problems::Table, Problems::SubjectId)
                            .to(Subjects::Table, Subjects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create problem_attempts table
        manager
            .create_table(
                Table::create()
                    .table(ProblemAttempts::Table)
                    .if_not_exists()
                    .col(uuid(ProblemAttempts::Id).primary_key())
                    .col(uuid(ProblemAttempts::ProblemId))
                    .col(integer(ProblemAttempts::TimeSpentSeconds))
                    .col(integer(ProblemAttempts::DifficultyRating))
                    .col(integer(ProblemAttempts::ConfidenceLevel))
                    .col(boolean(ProblemAttempts::WasSuccessful))
                    .col(string_null(ProblemAttempts::Notes))
                    .col(timestamp(ProblemAttempts::AttemptedAt))
                    .col(boolean(ProblemAttempts::IsSynced).default(false))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProblemAttempts::Table, ProblemAttempts::ProblemId)
                            .to(Problems::Table, Problems::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProblemAttempts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Problems::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Subjects::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Courses::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Folders::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Subscriptions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
    PasswordHash,
    Name,
    CreatedAt,
    UpdatedAt,
    IsPremium,
    LastSync,
}

#[derive(DeriveIden)]
enum Subscriptions {
    Table,
    Id,
    UserId,
    StripeCustomerId,
    StripeSubscriptionId,
    Status,
    CurrentPeriodStart,
    CurrentPeriodEnd,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Folders {
    Table,
    Id,
    UserId,
    Name,
    Description,
    SortOrder,
    CreatedAt,
    UpdatedAt,
    IsSynced,
}

#[derive(DeriveIden)]
enum Courses {
    Table,
    Id,
    FolderId,
    Name,
    Description,
    ColorCode,
    SortOrder,
    CreatedAt,
    UpdatedAt,
    IsSynced,
}

#[derive(DeriveIden)]
enum Subjects {
    Table,
    Id,
    CourseId,
    Name,
    Description,
    SortOrder,
    CreatedAt,
    UpdatedAt,
    IsSynced,
}

#[derive(DeriveIden)]
enum Problems {
    Table,
    Id,
    SubjectId,
    Title,
    Description,
    ImagePath,
    S3ImageKey,
    ConfidenceLevel,
    Notes,
    CreatedAt,
    UpdatedAt,
    LastAttempted,
    AttemptCount,
    SuccessRate,
    IsSynced,
    LastModified,
}

#[derive(DeriveIden)]
enum ProblemAttempts {
    Table,
    Id,
    ProblemId,
    TimeSpentSeconds,
    DifficultyRating,
    ConfidenceLevel,
    WasSuccessful,
    Notes,
    AttemptedAt,
    IsSynced,
}
