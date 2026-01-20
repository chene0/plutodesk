use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Represents an active study session with folder/course/set context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionState {
    pub id: Uuid,
    pub name: String,
    pub folder_id: Uuid,
    pub course_id: Uuid,
    pub set_id: Uuid,
    #[serde(with = "naive_datetime_format")]
    pub created_at: NaiveDateTime,
    #[serde(with = "naive_datetime_format")]
    pub last_used: NaiveDateTime,
}

// Custom serialization for NaiveDateTime to match database format
mod naive_datetime_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S%.f";

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

impl SessionState {
    pub fn new(
        name: String,
        folder_id: Uuid,
        course_id: Uuid,
        set_id: Uuid,
    ) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            id: Uuid::new_v4(),
            name,
            folder_id,
            course_id,
            set_id,
            created_at: now,
            last_used: now,
        }
    }

    pub fn update_last_used(&mut self) {
        self.last_used = Utc::now().naive_utc();
    }
}

/// Manages all saved sessions and tracks the active session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManager {
    pub sessions: Vec<SessionState>,
    pub active_session_id: Option<Uuid>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            active_session_id: None,
        }
    }

    /// Load session manager from JSON file
    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            log::info!("Session file does not exist, creating new SessionManager");
            return Ok(Self::new());
        }

        let contents = fs::read_to_string(path)?;
        let manager: SessionManager = serde_json::from_str(&contents)?;
        log::info!("Loaded {} sessions from file", manager.sessions.len());
        Ok(manager)
    }

    /// Save session manager to JSON file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        log::info!("Saved {} sessions to file", self.sessions.len());
        Ok(())
    }

    /// Get the currently active session
    pub fn get_active_session(&self) -> Option<&SessionState> {
        self.active_session_id
            .and_then(|id| self.sessions.iter().find(|s| s.id == id))
    }

    /// Get a mutable reference to the active session
    pub fn get_active_session_mut(&mut self) -> Option<&mut SessionState> {
        let active_id = self.active_session_id?;
        self.sessions.iter_mut().find(|s| s.id == active_id)
    }

    /// Start a session by ID
    pub fn start_session(&mut self, session_id: Uuid) -> Result<(), String> {
        // Check if session exists
        let session = self
            .sessions
            .iter_mut()
            .find(|s| s.id == session_id)
            .ok_or_else(|| format!("Session with id {} not found", session_id))?;

        // Update last used time
        session.update_last_used();
        self.active_session_id = Some(session_id);
        log::info!("Started session: {}", session.name);
        Ok(())
    }

    /// End the current session
    pub fn end_session(&mut self) {
        if let Some(id) = self.active_session_id {
            log::info!("Ended session with id: {}", id);
        }
        self.active_session_id = None;
    }

    /// Create a new session and optionally start it
    pub fn create_session(
        &mut self,
        name: String,
        folder_id: Uuid,
        course_id: Uuid,
        set_id: Uuid,
        start_immediately: bool,
    ) -> SessionState {
        let session = SessionState::new(name, folder_id, course_id, set_id);
        let session_id = session.id;
        self.sessions.push(session.clone());

        if start_immediately {
            self.active_session_id = Some(session_id);
        }

        log::info!("Created session: {}", session.name);
        session
    }

    /// Delete a session by ID
    pub fn delete_session(&mut self, session_id: Uuid) -> Result<(), String> {
        let index = self
            .sessions
            .iter()
            .position(|s| s.id == session_id)
            .ok_or_else(|| format!("Session with id {} not found", session_id))?;

        let session = self.sessions.remove(index);
        log::info!("Deleted session: {}", session.name);

        // If we deleted the active session, clear active_session_id
        if self.active_session_id == Some(session_id) {
            self.active_session_id = None;
        }

        Ok(())
    }

    /// Get all sessions
    pub fn get_all_sessions(&self) -> &[SessionState] {
        &self.sessions
    }

    /// Get a session by ID
    pub fn get_session_by_id(&self, session_id: Uuid) -> Option<&SessionState> {
        self.sessions.iter().find(|s| s.id == session_id)
    }

    /// Check if a session with the given folder/course/set combination already exists
    pub fn session_exists_for_context(&self, folder_id: Uuid, course_id: Uuid, set_id: Uuid) -> bool {
        self.sessions.iter().any(|s| {
            s.folder_id == folder_id && s.course_id == course_id && s.set_id == set_id
        })
    }
}

/// Wrapper for SessionManager to be used as Tauri state
pub type SessionManagerState = Arc<Mutex<SessionManager>>;


#[cfg(test)]
mod unit_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_session() {
        let mut manager = SessionManager::new();
        let folder_id = Uuid::new_v4();
        let course_id = Uuid::new_v4();
        let set_id = Uuid::new_v4();

        let session = manager.create_session(
            "Test Session".to_string(),
            folder_id,
            course_id,
            set_id,
            false,
        );

        assert_eq!(manager.sessions.len(), 1);
        assert_eq!(session.name, "Test Session");
        assert_eq!(session.folder_id, folder_id);
    }

    #[test]
    fn test_start_and_end_session() {
        let mut manager = SessionManager::new();
        let folder_id = Uuid::new_v4();
        let course_id = Uuid::new_v4();
        let set_id = Uuid::new_v4();

        let session = manager.create_session(
            "Test Session".to_string(),
            folder_id,
            course_id,
            set_id,
            false,
        );

        assert!(manager.get_active_session().is_none());

        manager.start_session(session.id).unwrap();
        assert!(manager.get_active_session().is_some());
        assert_eq!(manager.get_active_session().unwrap().id, session.id);

        manager.end_session();
        assert!(manager.get_active_session().is_none());
    }

    #[test]
    fn test_delete_session() {
        let mut manager = SessionManager::new();
        let folder_id = Uuid::new_v4();
        let course_id = Uuid::new_v4();
        let set_id = Uuid::new_v4();

        let session = manager.create_session(
            "Test Session".to_string(),
            folder_id,
            course_id,
            set_id,
            true,
        );

        assert_eq!(manager.sessions.len(), 1);
        assert!(manager.get_active_session().is_some());

        manager.delete_session(session.id).unwrap();
        assert_eq!(manager.sessions.len(), 0);
        assert!(manager.get_active_session().is_none());
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_sessions.json");

        // Create and save
        let mut manager = SessionManager::new();
        let folder_id = Uuid::new_v4();
        let course_id = Uuid::new_v4();
        let set_id = Uuid::new_v4();
        let folder_id_2 = Uuid::new_v4();

        manager.create_session(
            "Test Session 1".to_string(),
            folder_id,
            course_id,
            set_id,
            true,
        );
        manager.create_session(
            "Test Session 2".to_string(),
            folder_id_2,
            course_id,
            set_id,
            false,
        );

        manager.save_to_file(&file_path).unwrap();
        assert!(file_path.exists());

        // Load and verify
        let loaded_manager = SessionManager::load_from_file(&file_path).unwrap();
        assert_eq!(loaded_manager.sessions.len(), 2);
        assert!(loaded_manager.active_session_id.is_some());
        assert_eq!(
            loaded_manager.sessions[0].name,
            manager.sessions[0].name
        );
    }

    #[test]
    fn test_session_exists_for_context() {
        let mut manager = SessionManager::new();
        let folder_id = Uuid::new_v4();
        let course_id = Uuid::new_v4();
        let set_id = Uuid::new_v4();

        // No session exists initially
        assert!(!manager.session_exists_for_context(folder_id, course_id, set_id));

        // Create a session
        manager.create_session(
            "Test Session".to_string(),
            folder_id,
            course_id,
            set_id,
            false,
        );

        // Session should now exist for this context
        assert!(manager.session_exists_for_context(folder_id, course_id, set_id));

        // Different context should not exist
        let different_folder_id = Uuid::new_v4();
        assert!(!manager.session_exists_for_context(different_folder_id, course_id, set_id));
    }
}

