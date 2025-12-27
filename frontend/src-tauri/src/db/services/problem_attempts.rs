use sea_orm::*;
use crate::db::entities::{problem_attempts, problem_attempts::Entity as ProblemAttempt};
use uuid::Uuid;

pub async fn create_problem_attempt(
    db: &DatabaseConnection,
    problem_id: Uuid,
    time_spent_seconds: i32,
    difficulty_rating: i32,
    confidence_level: i32,
    was_successful: bool,
    notes: Option<String>,
) -> Result<problem_attempts::Model, DbErr> {
    let now = chrono::Utc::now().naive_utc();

    let attempt = problem_attempts::ActiveModel {
        id: Set(Uuid::new_v4()),
        problem_id: Set(problem_id),
        time_spent_seconds: Set(time_spent_seconds),
        difficulty_rating: Set(difficulty_rating),
        confidence_level: Set(confidence_level),
        was_successful: Set(was_successful),
        notes: Set(notes),
        attempted_at: Set(now),
        is_synced: Set(false),
    };

    attempt.insert(db).await
}

pub async fn get_attempt_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<problem_attempts::Model>, DbErr> {
    ProblemAttempt::find_by_id(id).one(db).await
}

pub async fn get_attempts_by_problem(
    db: &DatabaseConnection,
    problem_id: Uuid,
) -> Result<Vec<problem_attempts::Model>, DbErr> {
    ProblemAttempt::find()
        .filter(problem_attempts::Column::ProblemId.eq(problem_id))
        .order_by_desc(problem_attempts::Column::AttemptedAt)
        .all(db)
        .await
}

pub async fn update_attempt(
    db: &DatabaseConnection,
    id: Uuid,
    time_spent_seconds: Option<i32>,
    difficulty_rating: Option<i32>,
    confidence_level: Option<i32>,
    was_successful: Option<bool>,
    notes: Option<Option<String>>,
) -> Result<problem_attempts::Model, DbErr> {
    let attempt = ProblemAttempt::find_by_id(id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("Attempt not found".to_string()))?;

    let mut attempt: problem_attempts::ActiveModel = attempt.into();

    if let Some(ts) = time_spent_seconds {
        attempt.time_spent_seconds = Set(ts);
    }
    if let Some(dr) = difficulty_rating {
        attempt.difficulty_rating = Set(dr);
    }
    if let Some(cl) = confidence_level {
        attempt.confidence_level = Set(cl);
    }
    if let Some(ws) = was_successful {
        attempt.was_successful = Set(ws);
    }
    if let Some(n) = notes {
        attempt.notes = Set(n);
    }

    attempt.is_synced = Set(false);

    attempt.update(db).await
}

pub async fn delete_attempt(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<DeleteResult, DbErr> {
    ProblemAttempt::delete_by_id(id).exec(db).await
}
