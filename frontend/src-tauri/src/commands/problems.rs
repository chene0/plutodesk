use crate::db::{services, Db};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProblemRequest {
    pub subject_id: String,
    pub title: String,
    pub description: Option<String>,
    pub image_path: Option<String>,
    pub s3_image_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProblemRequest {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub image_path: Option<Option<String>>,
    pub s3_image_key: Option<Option<String>>,
    pub confidence_level: Option<i32>,
    pub notes: Option<Option<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProblemStatsRequest {
    pub id: String,
    pub was_successful: bool,
}

#[tauri::command]
pub async fn create_problem(
    db: State<'_, Db>,
    request: CreateProblemRequest,
) -> Result<String, String> {
    let subject_id = Uuid::parse_str(&request.subject_id).map_err(|e| e.to_string())?;

    let problem = services::create_problem(
        db.connection(),
        subject_id,
        request.title,
        request.description,
        request.image_path,
        request.s3_image_key,
    )
    .await
    .map_err(|e| e.to_string())?;

    serde_json::to_string(&problem).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_problem(db: State<'_, Db>, id: String) -> Result<String, String> {
    let problem_id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    let problem = services::get_problem_by_id(db.connection(), problem_id)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&problem).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_problems_by_subject(
    db: State<'_, Db>,
    subject_id: String,
) -> Result<String, String> {
    let subject_uuid = Uuid::parse_str(&subject_id).map_err(|e| e.to_string())?;

    let problems = services::get_problems_by_subject(db.connection(), subject_uuid)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&problems).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_problem(
    db: State<'_, Db>,
    request: UpdateProblemRequest,
) -> Result<String, String> {
    let problem_id = Uuid::parse_str(&request.id).map_err(|e| e.to_string())?;

    let problem = services::update_problem(
        db.connection(),
        problem_id,
        request.title,
        request.description,
        request.image_path,
        request.s3_image_key,
        request.confidence_level,
        request.notes,
    )
    .await
    .map_err(|e| e.to_string())?;

    serde_json::to_string(&problem).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_problem_stats(
    db: State<'_, Db>,
    request: UpdateProblemStatsRequest,
) -> Result<String, String> {
    let problem_id = Uuid::parse_str(&request.id).map_err(|e| e.to_string())?;

    let problem = services::update_problem_stats(
        db.connection(),
        problem_id,
        request.was_successful,
    )
    .await
    .map_err(|e| e.to_string())?;

    serde_json::to_string(&problem).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_problem(db: State<'_, Db>, id: String) -> Result<String, String> {
    let problem_id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    services::delete_problem(db.connection(), problem_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Problem deleted successfully".to_string())
}
