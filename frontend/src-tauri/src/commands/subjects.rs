use crate::db::{services, Db};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubjectRequest {
    pub course_id: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSubjectRequest {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub sort_order: Option<i32>,
}

#[tauri::command]
pub async fn create_subject(
    db: State<'_, Db>,
    request: CreateSubjectRequest,
) -> Result<String, String> {
    let course_id = Uuid::parse_str(&request.course_id).map_err(|e| e.to_string())?;

    let subject = services::create_subject(
        db.connection(),
        course_id,
        request.name,
        request.description,
        request.sort_order,
    )
    .await
    .map_err(|e| e.to_string())?;

    serde_json::to_string(&subject).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_subject(db: State<'_, Db>, id: String) -> Result<String, String> {
    let subject_id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    let subject = services::get_subject_by_id(db.connection(), subject_id)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&subject).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_subjects_by_course(
    db: State<'_, Db>,
    course_id: String,
) -> Result<String, String> {
    let course_uuid = Uuid::parse_str(&course_id).map_err(|e| e.to_string())?;

    let subjects = services::get_subjects_by_course(db.connection(), course_uuid)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&subjects).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_subject(
    db: State<'_, Db>,
    request: UpdateSubjectRequest,
) -> Result<String, String> {
    let subject_id = Uuid::parse_str(&request.id).map_err(|e| e.to_string())?;

    let subject = services::update_subject(
        db.connection(),
        subject_id,
        request.name,
        request.description,
        request.sort_order,
    )
    .await
    .map_err(|e| e.to_string())?;

    serde_json::to_string(&subject).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_subject(db: State<'_, Db>, id: String) -> Result<String, String> {
    let subject_id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    services::delete_subject(db.connection(), subject_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Subject deleted successfully".to_string())
}
