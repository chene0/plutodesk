use crate::db::{services, Db};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSetRequest {
    pub course_id: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSetRequest {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub sort_order: Option<i32>,
}

#[tauri::command]
pub async fn create_set(
    db: State<'_, Db>,
    request: CreateSetRequest,
) -> Result<String, String> {
    let course_id = Uuid::parse_str(&request.course_id).map_err(|e| e.to_string())?;

    let set = services::create_set(
        db.connection(),
        course_id,
        request.name,
        request.description,
        request.sort_order,
    )
    .await
    .map_err(|e| e.to_string())?;

    serde_json::to_string(&set).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_set(db: State<'_, Db>, id: String) -> Result<String, String> {
    let set_id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    let set = services::get_set_by_id(db.connection(), set_id)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&set).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sets_by_course(
    db: State<'_, Db>,
    course_id: String,
) -> Result<String, String> {
    let course_uuid = Uuid::parse_str(&course_id).map_err(|e| e.to_string())?;

    let sets = services::get_sets_by_course(db.connection(), course_uuid)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&sets).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_set(
    db: State<'_, Db>,
    request: UpdateSetRequest,
) -> Result<String, String> {
    let set_id = Uuid::parse_str(&request.id).map_err(|e| e.to_string())?;

    let set = services::update_set(
        db.connection(),
        set_id,
        request.name,
        request.description,
        request.sort_order,
    )
    .await
    .map_err(|e| e.to_string())?;

    serde_json::to_string(&set).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_set(db: State<'_, Db>, id: String) -> Result<String, String> {
    let set_id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    services::delete_set(db.connection(), set_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Set deleted successfully".to_string())
}
