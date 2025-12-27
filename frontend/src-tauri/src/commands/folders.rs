use crate::db::{services, Db};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFolderRequest {
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFolderRequest {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub sort_order: Option<i32>,
}

#[tauri::command]
pub async fn create_folder(
    db: State<'_, Db>,
    request: CreateFolderRequest,
) -> Result<String, String> {
    let user_id = Uuid::parse_str(&request.user_id).map_err(|e| e.to_string())?;

    let folder = services::create_folder(
        db.connection(),
        user_id,
        request.name,
        request.description,
        request.sort_order,
    )
    .await
    .map_err(|e| e.to_string())?;

    serde_json::to_string(&folder).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_folder(db: State<'_, Db>, id: String) -> Result<String, String> {
    let folder_id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    let folder = services::get_folder_by_id(db.connection(), folder_id)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&folder).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_folders_by_user(
    db: State<'_, Db>,
    user_id: String,
) -> Result<String, String> {
    let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;

    let folders = services::get_folders_by_user(db.connection(), user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&folders).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_folder(
    db: State<'_, Db>,
    request: UpdateFolderRequest,
) -> Result<String, String> {
    let folder_id = Uuid::parse_str(&request.id).map_err(|e| e.to_string())?;

    let folder = services::update_folder(
        db.connection(),
        folder_id,
        request.name,
        request.description,
        request.sort_order,
    )
    .await
    .map_err(|e| e.to_string())?;

    serde_json::to_string(&folder).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_folder(db: State<'_, Db>, id: String) -> Result<String, String> {
    let folder_id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    services::delete_folder(db.connection(), folder_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Folder deleted successfully".to_string())
}
