use crate::db::{services, Db};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCourseRequest {
    pub folder_id: String,
    pub name: String,
    pub description: Option<String>,
    pub color_code: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCourseRequest {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub color_code: Option<Option<String>>,
    pub sort_order: Option<i32>,
}

#[tauri::command]
pub async fn create_course(
    db: State<'_, Db>,
    request: CreateCourseRequest,
) -> Result<String, String> {
    let folder_id = Uuid::parse_str(&request.folder_id).map_err(|e| e.to_string())?;

    let course = services::create_course(
        db.connection(),
        folder_id,
        request.name,
        request.description,
        request.color_code,
        request.sort_order,
    )
    .await
    .map_err(|e| e.to_string())?;

    serde_json::to_string(&course).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_course(db: State<'_, Db>, id: String) -> Result<String, String> {
    let course_id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    let course = services::get_course_by_id(db.connection(), course_id)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&course).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_courses_by_folder(
    db: State<'_, Db>,
    folder_id: String,
) -> Result<String, String> {
    let folder_uuid = Uuid::parse_str(&folder_id).map_err(|e| e.to_string())?;

    let courses = services::get_courses_by_folder(db.connection(), folder_uuid)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&courses).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_course(
    db: State<'_, Db>,
    request: UpdateCourseRequest,
) -> Result<String, String> {
    let course_id = Uuid::parse_str(&request.id).map_err(|e| e.to_string())?;

    let course = services::update_course(
        db.connection(),
        course_id,
        request.name,
        request.description,
        request.color_code,
        request.sort_order,
    )
    .await
    .map_err(|e| e.to_string())?;

    serde_json::to_string(&course).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_course(db: State<'_, Db>, id: String) -> Result<String, String> {
    let course_id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    services::delete_course(db.connection(), course_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Course deleted successfully".to_string())
}
