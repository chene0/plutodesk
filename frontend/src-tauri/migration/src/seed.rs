use sea_orm::{DatabaseConnection, DbErr};
use sea_orm_migration::prelude::*;

/// Seed the database with sample data for development/testing
pub async fn seed(db: &DatabaseConnection) -> Result<(), DbErr> {
    use chrono::Utc;

    log::info!("Seeding database with test data...");

    // Clear existing data (in reverse order of foreign key dependencies)
    log::info!("Clearing existing data...");
    db.execute(db.get_database_backend().build(&Query::delete().from_table(ProblemAttempts::Table).to_owned()))
        .await?;
    db.execute(db.get_database_backend().build(&Query::delete().from_table(Problems::Table).to_owned()))
        .await?;
    db.execute(db.get_database_backend().build(&Query::delete().from_table(Sets::Table).to_owned()))
        .await?;
    db.execute(db.get_database_backend().build(&Query::delete().from_table(Courses::Table).to_owned()))
        .await?;
    db.execute(db.get_database_backend().build(&Query::delete().from_table(Folders::Table).to_owned()))
        .await?;
    db.execute(db.get_database_backend().build(&Query::delete().from_table(Users::Table).to_owned()))
        .await?;
    log::info!("Existing data cleared");

    // Create a test user
    let user_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().naive_utc();
    let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let insert_user = Query::insert()
        .into_table(Users::Table)
        .columns([
            Users::Id,
            Users::Email,
            Users::PasswordHash,
            Users::Name,
            Users::CreatedAt,
            Users::UpdatedAt,
            Users::IsPremium,
        ])
        .values_panic([
            user_id.clone().into(),
            "test@example.com".into(),
            "hashed_password_here".into(),
            "Test User".into(),
            now_str.clone().into(),
            now_str.clone().into(),
            false.into(),
        ])
        .to_owned();

    db.execute(db.get_database_backend().build(&insert_user))
        .await?;

    log::info!("Created test user: {}", user_id);

    // Create a test folder
    let folder_id = uuid::Uuid::new_v4().to_string();
    let insert_folder = Query::insert()
        .into_table(Folders::Table)
        .columns([
            Folders::Id,
            Folders::UserId,
            Folders::Name,
            Folders::Description,
            Folders::SortOrder,
            Folders::CreatedAt,
            Folders::UpdatedAt,
            Folders::IsSynced,
        ])
        .values_panic([
            folder_id.clone().into(),
            user_id.clone().into(),
            "Computer Science".into(),
            "CS courses and subjects".into(),
            0.into(),
            now_str.clone().into(),
            now_str.clone().into(),
            false.into(),
        ])
        .to_owned();

    db.execute(db.get_database_backend().build(&insert_folder))
        .await?;

    log::info!("Created test folder: Computer Science");

    // Create a test course
    let course_id = uuid::Uuid::new_v4().to_string();
    let insert_course = Query::insert()
        .into_table(Courses::Table)
        .columns([
            Courses::Id,
            Courses::FolderId,
            Courses::Name,
            Courses::Description,
            Courses::ColorCode,
            Courses::SortOrder,
            Courses::CreatedAt,
            Courses::UpdatedAt,
            Courses::IsSynced,
        ])
        .values_panic([
            course_id.clone().into(),
            folder_id.clone().into(),
            "Data Structures & Algorithms".into(),
            "Fundamental algorithms and data structures".into(),
            "#3B82F6".into(),
            0.into(),
            now_str.clone().into(),
            now_str.clone().into(),
            false.into(),
        ])
        .to_owned();

    db.execute(db.get_database_backend().build(&insert_course))
        .await?;

    log::info!("Created test course: Data Structures & Algorithms");

    // Create a test set
    let set_id = uuid::Uuid::new_v4().to_string();
    let insert_set = Query::insert()
        .into_table(Sets::Table)
        .columns([
            Sets::Id,
            Sets::CourseId,
            Sets::Name,
            Sets::Description,
            Sets::SortOrder,
            Sets::CreatedAt,
            Sets::UpdatedAt,
            Sets::IsSynced,
        ])
        .values_panic([
            set_id.clone().into(),
            course_id.clone().into(),
            "Binary Trees".into(),
            "Tree traversal and manipulation".into(),
            0.into(),
            now_str.clone().into(),
            now_str.clone().into(),
            false.into(),
        ])
        .to_owned();

    db.execute(db.get_database_backend().build(&insert_set))
        .await?;

    log::info!("Created test set: Binary Trees");

    // Create test problems
    let problem_ids = vec![
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
    ];

    let problems = vec![
        (
            "Inorder Traversal",
            "Implement inorder traversal of a binary tree",
        ),
        (
            "Find Height",
            "Calculate the height of a binary tree",
        ),
        (
            "Lowest Common Ancestor",
            "Find the LCA of two nodes in a binary tree",
        ),
    ];

    for (i, (title, description)) in problems.iter().enumerate() {
        let insert_problem = Query::insert()
            .into_table(Problems::Table)
            .columns([
                Problems::Id,
                Problems::SetId,
                Problems::Title,
                Problems::Description,
                Problems::ConfidenceLevel,
                Problems::CreatedAt,
                Problems::UpdatedAt,
                Problems::LastModified,
                Problems::AttemptCount,
                Problems::SuccessRate,
                Problems::IsSynced,
            ])
            .values_panic([
                problem_ids[i].clone().into(),
                set_id.clone().into(),
                (*title).into(),
                (*description).into(),
                0.into(),
                now_str.clone().into(),
                now_str.clone().into(),
                now_str.clone().into(),
                0.into(),
                0.0.into(),
                false.into(),
            ])
            .to_owned();

        db.execute(db.get_database_backend().build(&insert_problem))
            .await?;

        log::info!("Created test problem: {}", title);
    }

    // Create a test problem attempt for the first problem
    let attempt_id = uuid::Uuid::new_v4().to_string();
    let insert_attempt = Query::insert()
        .into_table(ProblemAttempts::Table)
        .columns([
            ProblemAttempts::Id,
            ProblemAttempts::ProblemId,
            ProblemAttempts::TimeSpentSeconds,
            ProblemAttempts::DifficultyRating,
            ProblemAttempts::ConfidenceLevel,
            ProblemAttempts::WasSuccessful,
            ProblemAttempts::Notes,
            ProblemAttempts::AttemptedAt,
            ProblemAttempts::IsSynced,
        ])
        .values_panic([
            attempt_id.clone().into(),
            problem_ids[0].clone().into(),
            300.into(),
            3.into(),
            4.into(),
            true.into(),
            "Solved using recursion".into(),
            now_str.clone().into(),
            false.into(),
        ])
        .to_owned();

    db.execute(db.get_database_backend().build(&insert_attempt))
        .await?;

    log::info!("Created test problem attempt");

    log::info!("Database seeding completed successfully!");
    Ok(())
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
enum Sets {
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
    SetId,
    Title,
    Description,
    ConfidenceLevel,
    CreatedAt,
    UpdatedAt,
    LastModified,
    AttemptCount,
    SuccessRate,
    IsSynced,
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
