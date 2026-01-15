use crate::db::{services, Db};
use crate::session::{SessionManagerState, SessionState};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub folder_name: String,
    pub course_name: String,
    pub subject_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: String,
    pub name: String,
    pub folder_id: String,
    pub course_id: String,
    pub subject_id: String,
    pub folder_name: String,
    pub course_name: String,
    pub subject_name: String,
    pub created_at: String,
    pub last_used: String,
}

impl SessionResponse {
    async fn from_session_state(
        session: &SessionState,
        db: &sea_orm::DatabaseConnection,
    ) -> Result<Self, String> {
        // Fetch folder, course, and subject names from database
        let folder = services::get_folder_by_id(db, session.folder_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Folder with id {} not found", session.folder_id))?;

        let course = services::get_course_by_id(db, session.course_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Course with id {} not found", session.course_id))?;

        let subject = services::get_subject_by_id(db, session.subject_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Subject with id {} not found", session.subject_id))?;

        Ok(SessionResponse {
            id: session.id.to_string(),
            name: session.name.clone(),
            folder_id: session.folder_id.to_string(),
            course_id: session.course_id.to_string(),
            subject_id: session.subject_id.to_string(),
            folder_name: folder.name,
            course_name: course.name,
            subject_name: subject.name,
            created_at: session.created_at.to_string(),
            last_used: session.last_used.to_string(),
        })
    }
}

/// Get all saved sessions
#[tauri::command]
pub async fn get_all_sessions(
    session_manager: State<'_, SessionManagerState>,
    db: State<'_, Db>,
) -> Result<Vec<SessionResponse>, String> {
    let sessions = {
        let manager = session_manager.lock().unwrap();
        manager.get_all_sessions().to_vec()
    };

    let mut responses = Vec::new();
    for session in sessions.iter() {
        let response = SessionResponse::from_session_state(session, db.connection()).await?;
        responses.push(response);
    }

    Ok(responses)
}

/// Get the currently active session
#[tauri::command]
pub async fn get_active_session(
    session_manager: State<'_, SessionManagerState>,
    db: State<'_, Db>,
) -> Result<Option<SessionResponse>, String> {
    let session_opt = {
        let manager = session_manager.lock().unwrap();
        manager.get_active_session().cloned()
    };

    if let Some(session) = session_opt {
        let response = SessionResponse::from_session_state(&session, db.connection()).await?;
        Ok(Some(response))
    } else {
        Ok(None)
    }
}

/// Start an existing session by ID
#[tauri::command]
pub async fn start_session(
    session_manager: State<'_, SessionManagerState>,
    app: AppHandle,
    session_id: String,
) -> Result<(), String> {
    let session_uuid = Uuid::parse_str(&session_id).map_err(|e| e.to_string())?;

    let mut manager = session_manager.lock().unwrap();
    manager.start_session(session_uuid)?;

    // Persist to file
    let sessions_path = get_sessions_file_path(&app)?;
    manager
        .save_to_file(&sessions_path)
        .map_err(|e| format!("Failed to save sessions: {}", e))?;

    Ok(())
}

/// Create a new session with folder/course/subject (creates entities if they don't exist)
#[tauri::command]
pub async fn create_and_start_session(
    session_manager: State<'_, SessionManagerState>,
    db: State<'_, Db>,
    app: AppHandle,
    request: CreateSessionRequest,
) -> Result<SessionResponse, String> {
    // Get or create default user
    let user_id = services::get_or_create_default_user(db.connection())
        .await
        .map_err(|e| e.to_string())?;

    // Find or create folder
    let folder =
        services::find_or_create_folder(db.connection(), user_id, request.folder_name.clone())
            .await
            .map_err(|e| e.to_string())?;

    // Find or create course
    let course =
        services::find_or_create_course(db.connection(), folder.id, request.course_name.clone())
            .await
            .map_err(|e| e.to_string())?;

    // Find or create subject
    let subject =
        services::find_or_create_subject(db.connection(), course.id, request.subject_name.clone())
            .await
            .map_err(|e| e.to_string())?;

    // Auto-generate session name from folder/course/subject
    let session_name = format!(
        "{} / {} / {}",
        request.folder_name, request.course_name, request.subject_name
    );

    // Create session
    let session = {
        let mut manager = session_manager.lock().unwrap();

        // Check if a session with the same folder/course/subject already exists
        if manager.session_exists_for_context(folder.id, course.id, subject.id) {
            return Err(format!(
                "A session for '{}' already exists. Please select a different folder/course/subject combination.",
                session_name
            ));
        }

        let session = manager.create_session(
            session_name,
            folder.id,
            course.id,
            subject.id,
            true, // Start immediately
        );

        // Persist to file
        let sessions_path = get_sessions_file_path(&app)?;
        manager
            .save_to_file(&sessions_path)
            .map_err(|e| format!("Failed to save sessions: {}", e))?;

        session
    };

    // Create response
    let response = SessionResponse::from_session_state(&session, db.connection()).await?;
    Ok(response)
}

/// End the current session
#[tauri::command]
pub async fn end_session(
    session_manager: State<'_, SessionManagerState>,
    app: AppHandle,
) -> Result<(), String> {
    let mut manager = session_manager.lock().unwrap();
    manager.end_session();

    // Persist to file
    let sessions_path = get_sessions_file_path(&app)?;
    manager
        .save_to_file(&sessions_path)
        .map_err(|e| format!("Failed to save sessions: {}", e))?;

    Ok(())
}

/// Delete a saved session
#[tauri::command]
pub async fn delete_session(
    session_manager: State<'_, SessionManagerState>,
    app: AppHandle,
    session_id: String,
) -> Result<(), String> {
    let session_uuid = Uuid::parse_str(&session_id).map_err(|e| e.to_string())?;

    let mut manager = session_manager.lock().unwrap();
    manager.delete_session(session_uuid)?;

    // Persist to file
    let sessions_path = get_sessions_file_path(&app)?;
    manager
        .save_to_file(&sessions_path)
        .map_err(|e| format!("Failed to save sessions: {}", e))?;

    Ok(())
}

/// Helper function to get the sessions file path
fn get_sessions_file_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    Ok(app_data_dir.join("sessions.json"))
}

#[cfg(test)]
#[path = "sessions_test.rs"]
mod sessions_test;
