# Session Management Implementation

## Overview

This document describes the implementation of session-based workflow for PlutoDesk, where users must start a session before taking screenshots. All screenshots are automatically saved to the folder/course/subject context associated with the active session.

## Architecture

### Backend Components

#### 1. Session State Module (`frontend/src-tauri/src/session/mod.rs`)

**Key Structures:**
- `SessionState`: Represents a single session with folder/course/subject context
- `SessionManager`: Manages all sessions and tracks the active session
- `SessionManagerState`: Thread-safe wrapper for Tauri state management

**Features:**
- Session persistence to JSON file
- CRUD operations for sessions
- Active session tracking
- Last used timestamp updates

#### 2. Session Commands (`frontend/src-tauri/src/commands/sessions.rs`)

**Exposed Tauri Commands:**
- `get_all_sessions()`: Returns all saved sessions with entity names
- `get_active_session()`: Returns currently active session or null
- `start_session(session_id)`: Activates an existing session
- `create_and_start_session(request)`: Creates new session with folder/course/subject
- `end_session()`: Deactivates current session
- `delete_session(session_id)`: Removes a saved session

#### 3. Updated Screenshot Flow (`frontend/src-tauri/src/screenshot.rs`)

**Changes:**
- `receive_screenshot_data` now accepts optional folder_id, course_id, subject_id
- If IDs not provided, uses active session context
- Returns error if no active session and no IDs provided
- Fetches entity names from database for filesystem path creation

#### 4. System Tray Integration (`frontend/src-tauri/src/tray.rs`)

**Features:**
- "Start/Switch Session" menu item → Opens session modal
- "Active Session: [name]" label → Shows current session
- "End Session" menu item → Deactivates session
- "Quit" menu item → Exits application

### Frontend Components

#### 1. Session Modal (`frontend/src/components/session-modal.tsx`)

**Features:**
- Lists all saved sessions with folder > course > subject path
- Shows currently active session
- "New Session" form with hybrid input (select existing or type new)
- Session deletion with confirmation
- Real-time dropdown population based on selections
- Listens for `open-session-modal` event from tray

#### 2. Updated Screenshot Overlay (`frontend/src/components/screenshot-overlay.tsx`)

**Changes:**
- Checks for active session before saving
- If session exists: Prompts only for problem name
- If no session: Prompts user to start session first
- Emits `open-session-modal` event when needed

## Data Flow

```
User → System Tray → Session Modal → Session Manager → sessions.json
                                    ↓
                              Active Session
                                    ↓
Screenshot Hotkey → Screenshot Overlay → Check Session → Save with Context
                                                        ↓
                                                   Database + Filesystem
```

## File Structure

### New Files
- `frontend/src-tauri/src/session/mod.rs` - Session state and manager
- `frontend/src-tauri/src/session/tests.rs` - Session integration tests
- `frontend/src-tauri/src/commands/sessions.rs` - Session Tauri commands
- `frontend/src-tauri/src/tray.rs` - System tray implementation
- `frontend/src-tauri/src/screenshot_session_test.rs` - Screenshot + session tests
- `frontend/src/components/session-modal.tsx` - Session UI component
- `e2e/tauri/session-flow.test.ts` - E2E test scenarios

### Modified Files
- `frontend/src-tauri/src/lib.rs` - Added session state initialization and tray
- `frontend/src-tauri/src/screenshot.rs` - Updated to use session context
- `frontend/src-tauri/src/dtos/screenshot.rs` - No changes needed (still uses names)
- `frontend/src-tauri/src/commands/mod.rs` - Exported session commands
- `frontend/src-tauri/src/db/services/screenshots.rs` - Made helper functions public
- `frontend/src/app/layout.tsx` - Added SessionModal component

## Session Persistence

Sessions are stored in `sessions.json` in the app data directory:

```json
{
  "sessions": [
    {
      "id": "uuid",
      "name": "CS Algorithms Study",
      "folder_id": "uuid",
      "course_id": "uuid",
      "subject_id": "uuid",
      "created_at": "2026-01-04 12:00:00.000",
      "last_used": "2026-01-04 13:30:00.000"
    }
  ],
  "active_session_id": "uuid or null"
}
```

## User Workflow

### Starting a Session

1. User clicks system tray icon
2. Selects "Start/Switch Session" from menu
3. Session modal opens
4. User clicks "New Session"
5. Fills in:
   - Session name (e.g., "CS Study")
   - Folder (select existing or type new)
   - Course (select existing or type new)
   - Subject (select existing or type new)
6. Clicks "Create & Start Session"
7. Session becomes active
8. Tray menu shows "Active: CS Study"

### Taking Screenshots with Active Session

1. User presses Ctrl+Shift+S (or Cmd+Shift+S)
2. Screenshot overlay appears
3. User selects area
4. Prompt asks for problem name only
5. Screenshot saves to active session's folder/course/subject
6. Problem is created in database with correct subject_id

### Taking Screenshots without Active Session

1. User presses screenshot hotkey
2. Screenshot overlay appears
3. User selects area
4. Alert prompts: "Please start a session from the system tray menu"
5. Overlay closes without saving

### Switching Sessions

1. User opens session modal from tray
2. Clicks "Start" on a different saved session
3. That session becomes active
4. All subsequent screenshots save to new session context

### Ending a Session

1. User clicks "End Session" in tray menu
2. Active session is cleared
3. Session remains in saved sessions list
4. User must start a session before taking more screenshots

## Testing

### Unit Tests
- Session creation, deletion, switching
- Session persistence (load/save)
- Last used timestamp updates
- Active session tracking

### Integration Tests
- Screenshot save with active session
- Screenshot save creates correct hierarchy
- Session switch affects screenshot location
- Multiple screenshots in same session

### E2E Test Scenarios
- Complete session creation workflow
- Session switching between multiple sessions
- Session persistence across app restarts
- Screenshot flow with and without active session
- System tray interaction
- Session modal UI interactions

## Edge Cases Handled

1. **No Active Session**: User is prompted to start a session
2. **Deleted Entities**: Session start validates that folder/course/subject still exist
3. **Corrupted sessions.json**: App starts with empty SessionManager
4. **Filesystem Errors**: Errors are logged and user receives feedback
5. **Deleting Active Session**: Active session ID is cleared
6. **App Restart**: Active session is restored from sessions.json

## Future Enhancements

1. **Session Templates**: Pre-configure common folder/course/subject combinations
2. **Quick Switch**: Keyboard shortcut to switch between recent sessions
3. **Session Statistics**: Show number of problems per session
4. **Session Export**: Export all problems from a session
5. **Session Archiving**: Archive old sessions without deleting
6. **Session Sharing**: Share session configurations with other users
7. **Auto-Session**: Automatically suggest session based on time/day patterns

## Dependencies

- `chrono`: DateTime handling
- `serde`: Serialization/deserialization
- `uuid`: Unique identifiers
- `tauri`: System tray, state management, commands
- `sea-orm`: Database operations

## Configuration

No additional configuration required. Sessions are automatically persisted to:
- Windows: `%APPDATA%\com.plutodesk.desktop\sessions.json`
- macOS: `~/Library/Application Support/com.plutodesk.desktop/sessions.json`
- Linux: `~/.local/share/com.plutodesk.desktop/sessions.json`

