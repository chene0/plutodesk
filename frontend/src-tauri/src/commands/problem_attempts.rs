use crate::db::{services, Db};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAttemptRequest {
    pub problem_id: String,
    pub time_spent_seconds: i32,
    pub difficulty_rating: i32,
    pub confidence_level: i32,
    pub was_successful: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAttemptRequest {
    pub id: String,
    pub time_spent_seconds: Option<i32>,
    pub difficulty_rating: Option<i32>,
    pub confidence_level: Option<i32>,
    pub was_successful: Option<bool>,
    pub notes: Option<Option<String>>,
}

#[tauri::command]
pub async fn create_problem_attempt(
    db: State<'_, Db>,
    request: CreateAttemptRequest,
) -> Result<String, String> {
    let problem_id = Uuid::parse_str(&request.problem_id).map_err(|e| e.to_string())?;

    let attempt = services::create_problem_attempt(
        db.connection(),
        problem_id,
        request.time_spent_seconds,
        request.difficulty_rating,
        request.confidence_level,
        request.was_successful,
        request.notes,
    )
    .await
    .map_err(|e| e.to_string())?;

    serde_json::to_string(&attempt).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_problem_attempt(db: State<'_, Db>, id: String) -> Result<String, String> {
    let attempt_id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    let attempt = services::get_attempt_by_id(db.connection(), attempt_id)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&attempt).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_attempts_by_problem(
    db: State<'_, Db>,
    problem_id: String,
) -> Result<String, String> {
    let problem_uuid = Uuid::parse_str(&problem_id).map_err(|e| e.to_string())?;

    let attempts = services::get_attempts_by_problem(db.connection(), problem_uuid)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&attempts).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_problem_attempt(
    db: State<'_, Db>,
    request: UpdateAttemptRequest,
) -> Result<String, String> {
    let attempt_id = Uuid::parse_str(&request.id).map_err(|e| e.to_string())?;

    let attempt = services::update_attempt(
        db.connection(),
        attempt_id,
        request.time_spent_seconds,
        request.difficulty_rating,
        request.confidence_level,
        request.was_successful,
        request.notes,
    )
    .await
    .map_err(|e| e.to_string())?;

    serde_json::to_string(&attempt).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_problem_attempt(db: State<'_, Db>, id: String) -> Result<String, String> {
    let attempt_id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    services::delete_attempt(db.connection(), attempt_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Attempt deleted successfully".to_string())
}
