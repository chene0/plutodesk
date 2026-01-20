use crate::db::{services, Db};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProblemRequest {
    pub set_id: String,
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
    let set_id = Uuid::parse_str(&request.set_id).map_err(|e| e.to_string())?;

    let problem = services::create_problem(
        db.connection(),
        set_id,
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
pub async fn get_problems_by_set(
    db: State<'_, Db>,
    set_id: String,
) -> Result<String, String> {
    let set_uuid = Uuid::parse_str(&set_id).map_err(|e| e.to_string())?;

    let problems = services::get_problems_by_set(db.connection(), set_uuid)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_parsing_validation() {
        // Test valid UUID
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(Uuid::parse_str(valid_uuid).is_ok());

        // Test invalid UUID formats
        assert!(Uuid::parse_str("not-a-uuid").is_err());
        assert!(Uuid::parse_str("").is_err());
        assert!(Uuid::parse_str("550e8400-e29b-41d4-a716").is_err());
    }

    #[test]
    fn test_create_problem_request_validation() {
        let request = CreateProblemRequest {
            set_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            title: "Test Problem".to_string(),
            description: Some("Test Description".to_string()),
            image_path: None,
            s3_image_key: None,
        };

        // Should parse UUID successfully
        assert!(Uuid::parse_str(&request.set_id).is_ok());
    }

    #[test]
    fn test_update_problem_request_validation() {
        let request = UpdateProblemRequest {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            title: Some("Updated Title".to_string()),
            description: Some(Some("Updated Description".to_string())),
            image_path: None,
            s3_image_key: None,
            confidence_level: Some(5),
            notes: Some(Some("Some notes".to_string())),
        };

        // Should parse UUID successfully
        assert!(Uuid::parse_str(&request.id).is_ok());
    }
}
