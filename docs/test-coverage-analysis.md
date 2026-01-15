# Test Coverage Analysis - Post PR #2

## Summary

This document analyzes test coverage for all commits since PR #2 was merged (commit `5f1e819`). The following commits were reviewed:

1. **dae1fed** - fix(screenshot): resolve production black screen via Blob/ImageBitmap
2. **959f218** - fix: prevent screenshot overlay from closing immediately after reopen
3. **85f0e6a** - implement basic session management (#3)
4. **931a1d8** - Move check for existence of a session immediately after keybind
5. **a3c45c0** - get backend to resolve all-zeros uuid to default user id

## Current Test Coverage

### Existing Tests
- ✅ `screenshot-overlay.test.tsx` - Frontend component tests (comprehensive)
- ✅ `screenshot_integration_test.rs` - Database integration tests
- ✅ `screenshot_session_test.rs` - Session-screenshot integration tests
- ✅ `session/mod.rs` - Unit tests for SessionManager
- ✅ `screenshot.rs` - Unit tests for name sanitization and base64 handling
- ✅ E2E test scaffolds (not yet implemented)

### Test Coverage Gaps

---

## 1. Blob/ImageBitmap Screenshot Handling (Commit: dae1fed)

### What Changed
- Replaced Image+dataURL approach with Blob→ImageBitmap
- Added `get_screenshot_data` command to fetch screenshot from state
- Implemented ScreenshotData state management
- Fixed canvas CSS sizing conflicts

### Missing Tests

#### Frontend Tests (React/TypeScript)
**File: `frontend/src/__tests__/components/screenshot-overlay-blob.test.tsx`**

```typescript
// Test cases needed:
1. Test Blob creation from base64 string
   - Valid base64 input
   - Invalid base64 input
   - Empty base64 string

2. Test createImageBitmap success and failure
   - Successful bitmap creation
   - Bitmap creation failure (corrupted data)
   - Timeout scenarios

3. Test large screenshot handling
   - Screenshots > 2MB (the original problem this commit fixed)
   - Screenshots > 10MB
   - Memory constraints

4. Test canvas resizing with ImageBitmap
   - Canvas dimensions match bitmap dimensions
   - Canvas context properly draws bitmap
   - Canvas is properly cleared between screenshots

5. Test error handling in handleScreenshotData
   - atob() failures
   - Blob creation failures
   - createImageBitmap() rejections
   - Canvas context unavailable
```

#### Rust Tests
**File: `frontend/src-tauri/src/screenshot.rs`**

```rust
// Test cases needed:
1. Test get_screenshot_data command
   - Returns data when present
   - Returns None when data is empty
   - Thread-safe access to Mutex

2. Test ScreenshotData state initialization
   - State is properly initialized on app startup
   - State persists across commands
   - State is properly cleaned up

3. Test screenshot data storage
   - Data is stored before window creation
   - Data persists until window retrieves it
   - Data can be retrieved multiple times
```

---

## 2. Window Lifecycle Management (Commit: 959f218)

### What Changed
- Added window validity checks (`is_visible()`)
- Cleanup of stale/hidden windows
- Event listener cleanup on window close
- Proper Window instance handling in frontend

### Missing Tests

#### Frontend Tests
**File: `frontend/src/__tests__/components/screenshot-overlay-lifecycle.test.tsx`**

```typescript
// Test cases needed:
1. Test Window instance creation
   - Window is created only once per component instance
   - Window reference persists across re-renders
   - Window reference is cleared on unmount

2. Test window reference staleness
   - Detect stale Window instances
   - Handle errors from invalid Window instances
   - Gracefully degrade when Window operations fail

3. Test event listener cleanup
   - Listeners are removed on unmount
   - No memory leaks from dangling listeners
   - Multiple mount/unmount cycles work correctly

4. Test rapid open/close scenarios
   - Opening overlay multiple times in quick succession
   - Closing before fully opened
   - Opening while closing
```

#### Rust Tests
**File: `frontend/src-tauri/src/screenshot_window_lifecycle_test.rs`** (NEW)

```rust
// Test cases needed:
1. Test window existence checks
   - get_webview_window() returns existing window
   - is_visible() returns correct state
   - Handle windows in invalid states

2. Test stale window cleanup
   - Hidden windows are closed before creating new ones
   - Invalid state windows are closed
   - Cleanup waits appropriate duration

3. Test listener cleanup on window close
   - screenshot_overlay_ready listener is removed
   - Listeners are removed when window closes
   - No listener leaks across multiple sessions

4. Test window creation after cleanup
   - New window can be created after closing previous
   - State is properly reset between windows
   - No race conditions between close and create

5. Test fallback window show logic
   - Window is shown after 500ms if still hidden
   - Fallback checks window validity before showing
   - Fallback handles window already visible
   - Fallback handles window closed/invalid
```

---

## 3. Session Management System (Commit: 85f0e6a, 931a1d8)

### What Changed
- Complete session management system
- SessionManager with CRUD operations
- Session persistence to JSON file
- Session commands (create, start, end, delete, get)
- Integration with screenshot workflow
- Tray menu integration
- Session modal UI component

### Missing Tests

#### Backend - Session Commands
**File: `frontend/src-tauri/src/commands/sessions_test.rs`** (NEW)

```rust
// Test cases needed:
1. Test create_and_start_session
   - Creates new folder/course/subject if they don't exist
   - Reuses existing folder/course/subject if they exist
   - Prevents duplicate session creation (931a1d8)
   - Returns appropriate error messages
   - Persists session to file

2. Test duplicate session prevention
   - Detects existing session with same folder/course/subject
   - Returns user-friendly error message
   - Check happens before database operations
   - Check happens immediately after keybind (931a1d8)

3. Test start_session
   - Updates last_used timestamp
   - Sets active_session_id correctly
   - Persists changes to file
   - Handles non-existent session ID

4. Test end_session
   - Clears active_session_id
   - Session remains in saved sessions
   - Persists changes to file

5. Test delete_session
   - Removes session from list
   - Clears active_session_id if deleting active session
   - Persists changes to file
   - Handles non-existent session ID

6. Test get_all_sessions
   - Returns all sessions with names resolved
   - Handles database lookup failures gracefully
   - Returns empty array when no sessions exist

7. Test get_active_session
   - Returns active session with names resolved
   - Returns None when no active session
   - Handles database lookup failures

8. Test UUID parsing errors
   - Invalid UUID format
   - Empty UUID string
   - Malformed UUID strings

9. Test database lookup failures
   - Folder not found (deleted from DB)
   - Course not found (deleted from DB)
   - Subject not found (deleted from DB)

10. Test file I/O errors
    - Unable to read sessions.json
    - Unable to write sessions.json
    - Corrupted sessions.json
    - Directory doesn't exist
```

#### Backend - UUID Resolution
**File: `frontend/src-tauri/src/commands/folders_test.rs`** (EXPAND)

```rust
// Test cases needed (for commit a3c45c0):
1. Test all-zeros UUID resolution
   - "00000000-0000-0000-0000-000000000000" resolves to default user
   - Default user is created if doesn't exist
   - Default user is reused if exists

2. Test normal UUID passthrough
   - Valid UUID is parsed and used directly
   - UUID is not modified

3. Test invalid UUID handling
   - Returns appropriate error message
   - Doesn't crash application
```

#### Frontend - Session Modal UI
**File: `frontend/src/__tests__/components/session-modal.test.tsx`** (NEW)

```typescript
// Test cases needed (for commit a3c45c0):
1. Test dropdown mode vs create mode
   - Can select existing folder from dropdown
   - Can switch to "Create new folder" mode
   - Can switch back to dropdown from create mode
   - State is properly reset when switching modes

2. Test create new folder flow
   - Input field appears in create mode
   - Back button returns to dropdown
   - Validation for empty name
   - Validation for duplicate name

3. Test create new course flow
   - Dropdown is disabled until folder selected
   - Create mode shows input field
   - Validation for empty/duplicate name

4. Test create new subject flow
   - Dropdown is disabled until course selected
   - Create mode shows input field
   - Validation for empty/duplicate name

5. Test duplicate name validation
   - Detects duplicate folder name (case-insensitive)
   - Detects duplicate course name (case-insensitive)
   - Detects duplicate subject name (case-insensitive)
   - Shows user-friendly error message
   - Error is cleared when input changes

6. Test form state reset
   - All fields cleared on cancel
   - All mode flags reset on cancel
   - Error message cleared on cancel
   - Selected IDs cleared appropriately

7. Test cascade dropdowns
   - Selecting folder loads courses for that folder
   - Selecting course loads subjects for that course
   - Changing folder clears course and subject
   - Changing course clears subject

8. Test session creation validation
   - All fields must be filled
   - Error shown if any field empty
   - Can create with all dropdown selections
   - Can create with all new inputs
   - Can create with mixed dropdown/new inputs

9. Test saved sessions list
   - Displays all saved sessions
   - Shows folder > course > subject hierarchy
   - Highlights active session
   - Delete button per session
   - Start button per session

10. Test session deletion
    - Shows confirmation dialog
    - Deletes session on confirm
    - Cancels deletion on cancel
    - Updates UI after deletion
    - Clears active session if deleting active
```

---

## 4. Notification & Main Window Management (Commit: 85f0e6a)

### What Changed
- `check_session_and_notify` function
- `open_or_focus_main_window_with_session_modal` function
- OS notification when no active session
- Main window creation or focusing

### Missing Tests

**File: `frontend/src-tauri/src/screenshot_session_integration_test.rs`** (EXPAND)

```rust
// Test cases needed:
1. Test check_session_and_notify
   - Returns true when session active
   - Returns false when no session
   - Shows notification when no session
   - Opens/focuses main window when no session
   - Emits open-session-modal event

2. Test main window management
   - Creates window if doesn't exist
   - Focuses existing window if exists
   - Sets correct window properties
   - Successfully shows and focuses window

3. Test notification display
   - Notification contains correct title
   - Notification contains correct body
   - Handles notification API failures gracefully
```

---

## 5. Screenshot-Session Integration

### What Changed
- Screenshot save now requires active session OR explicit folder/course/subject IDs
- Session check happens before screenshot overlay opens
- Problem name prompt instead of full hierarchy prompt

### Missing Tests

**File: `frontend/src-tauri/src/screenshot_session_integration_test.rs`** (EXPAND)

```rust
// Test cases needed:
1. Test screenshot blocked without session
   - check_session_and_notify returns false
   - Screenshot overlay doesn't open
   - Notification is shown
   - Main window opens with modal

2. Test screenshot with inline session selection
   - receive_screenshot_data with folder_id/course_id/subject_id
   - Validates all UUIDs
   - Fetches folder/course/subject names from DB
   - Saves screenshot to correct location

3. Test screenshot with active session
   - receive_screenshot_data without IDs uses active session
   - Fetches folder/course/subject from active session
   - Saves screenshot to session context

4. Test database lookup failures
   - Folder ID not found
   - Course ID not found
   - Subject ID not found
   - Returns user-friendly error

5. Test invalid UUID parameters
   - Invalid folder_id
   - Invalid course_id
   - Invalid subject_id
   - Returns parse error
```

**File: `frontend/src/__tests__/components/screenshot-overlay-session.test.tsx`** (NEW)

```typescript
// Test cases needed:
1. Test problem name prompt
   - Prompt appears when session active
   - Prompt has only problem name field
   - No folder/course/subject prompts
   - Validation for empty problem name

2. Test screenshot save with session
   - Invokes receive_screenshot_data with problem name only
   - No folder/course/subject IDs passed
   - Backend uses active session context

3. Test screenshot save with inline selection
   - Invokes receive_screenshot_data with all IDs
   - Folder/course/subject IDs are passed
   - Backend uses provided IDs instead of session
```

---

## 6. System Tray Integration

### What Changed
- Tray menu with Start/Switch Session, End Session, Quit
- Tray menu event handlers
- Session modal opening from tray

### Missing Tests

**File: `frontend/src-tauri/src/tray_test.rs`** (NEW)

```rust
// Test cases needed:
1. Test tray menu creation
   - All menu items created correctly
   - Menu items have correct IDs
   - Menu items are enabled

2. Test "Start/Switch Session" menu item
   - Emits open-session-modal event
   - Event is received by frontend

3. Test "End Session" menu item
   - Calls session_manager.end_session()
   - Persists changes to file
   - Handles errors gracefully

4. Test "Quit" menu item
   - Application exits with code 0
   - Cleanup happens before exit

5. Test event handler errors
   - Handles state access failures
   - Handles file I/O failures
   - Logs errors appropriately
```

---

## 7. E2E Test Implementation

### Current Status
- E2E test scaffolds exist but are not implemented
- Tests are documented but not executable

### Missing Implementation

**File: `e2e/tauri/screenshot-flow.test.ts`** (IMPLEMENT)
- All documented test scenarios need implementation
- Requires Tauri test harness setup
- Needs screenshot capture and verification

**File: `e2e/tauri/session-flow.test.ts`** (IMPLEMENT)
- All documented test scenarios need implementation
- Requires Tauri test harness setup
- Needs system tray interaction support
- Needs filesystem verification (sessions.json)

---

## 8. Additional Test Categories

### Error Handling & Edge Cases

**File: `frontend/src-tauri/src/error_handling_test.rs`** (NEW)

```rust
// Test cases needed:
1. Test concurrent operations
   - Multiple screenshot captures in quick succession
   - Session operations during screenshot capture
   - Database operations during session operations

2. Test resource cleanup
   - Memory leaks from repeated operations
   - File handles properly closed
   - Database connections properly managed

3. Test invalid state transitions
   - Delete active session during screenshot
   - End session during screenshot save
   - Switch session during screenshot capture

4. Test filesystem errors
   - Disk full when saving screenshot
   - Permission denied when creating directory
   - Read-only filesystem

5. Test database errors
   - Connection lost during operation
   - Transaction rollback scenarios
   - Constraint violations
```

### Performance Tests

**File: `frontend/src-tauri/benches/screenshot_bench.rs`** (NEW)

```rust
// Benchmarks needed:
1. Screenshot capture time
   - Different monitor resolutions
   - Multiple monitors

2. Base64 encoding time
   - Various image sizes

3. Blob→ImageBitmap conversion time (frontend)
   - Various image sizes

4. Session operations
   - Create session
   - Load sessions from file
   - Save sessions to file

5. Database operations
   - Find or create folder/course/subject
   - Save screenshot to DB
```

---

## Priority Recommendations

### High Priority (Must Have)
1. **Session Commands Tests** - Critical business logic, many edge cases
2. **UUID Resolution Tests** - Security and user experience
3. **Duplicate Session Prevention** - User experience and data integrity
4. **Window Lifecycle Tests** - Prevents race conditions and crashes
5. **Session-Screenshot Integration** - Core workflow validation

### Medium Priority (Should Have)
1. **Blob/ImageBitmap Tests** - Ensures fix for production black screen stays fixed
2. **Session Modal UI Tests** - Complex UI with validation logic
3. **Error Handling Tests** - Robustness and user experience
4. **Notification Tests** - User experience

### Low Priority (Nice to Have)
1. **E2E Test Implementation** - Time-consuming but valuable
2. **Performance Benchmarks** - Optimization reference
3. **Tray Menu Tests** - Simple logic, less likely to break

---

## Test Implementation Strategy

### Phase 1: Critical Business Logic (Week 1)
- Session commands tests
- UUID resolution tests
- Duplicate session prevention tests
- Session-screenshot integration tests

### Phase 2: Window & Lifecycle (Week 2)
- Window lifecycle tests
- Blob/ImageBitmap tests
- Event listener cleanup tests

### Phase 3: UI & UX (Week 3)
- Session modal UI tests
- Error handling tests
- Notification tests

### Phase 4: E2E & Performance (Week 4)
- E2E test implementation
- Performance benchmarks
- Tray menu tests

---

## Estimated Test Count

| Category | Test Files | Approximate Test Cases |
|----------|-----------|------------------------|
| Session Commands | 1 | 25-30 |
| UUID Resolution | 1 (expand existing) | 5-7 |
| Window Lifecycle | 1 | 15-20 |
| Blob/ImageBitmap | 1 | 10-15 |
| Session Modal UI | 1 | 30-40 |
| Screenshot-Session | 1 (expand existing) | 15-20 |
| Tray Integration | 1 | 8-10 |
| Error Handling | 1 | 10-15 |
| E2E Implementation | 2 | 40-50 |
| Performance Benchmarks | 1 | 5-8 |
| **TOTAL** | **11 files** | **~165-220 tests** |

---

## Conclusion

The commits since PR #2 introduced significant new functionality, particularly around:
1. **Session management system** - Complete CRUD with persistence
2. **Window lifecycle management** - Preventing race conditions
3. **Blob/ImageBitmap handling** - Fixing production screenshot issues
4. **Session-screenshot integration** - Core workflow changes

These areas have minimal to no test coverage and represent the highest risk for regressions. The recommended approach is to implement tests in phases, prioritizing critical business logic first, followed by window/lifecycle management, then UI/UX, and finally E2E and performance tests.

**Current Test Coverage Estimate: ~25%**
**Target Test Coverage: ~80%**
**Gap: ~55% (165-220 additional tests needed)**
