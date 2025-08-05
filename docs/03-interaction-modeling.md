# 3. Interaction Modeling

## Component Interactions
- Screenshot capture and problem logging (Desktop only)
- Manual image upload and problem creation (Web and Desktop)
- Problem organization into hierarchical structure (Folders → Courses → Subjects)
- Performance tracking and analytics
- Cloud sync for premium users
- Revision mode with filtered problem sets

## Key User Flows

### 1. Problem Creation (Desktop - Screenshot)
```mermaid
sequenceDiagram
    participant User
    participant Desktop
    participant SQLite
    participant FileSystem
    participant Backend
    participant PostgreSQL
    participant S3
    
    User->>Desktop: Press keybind for screenshot
    Desktop->>Desktop: Capture screenshot
    Desktop->>Desktop: Show overlay UI
    User->>Desktop: Fill problem details (subject, difficulty, notes)
    Desktop->>FileSystem: Save image to local directory
    FileSystem-->>Desktop: Return file path
    Desktop->>SQLite: Store problem metadata with file path
    SQLite-->>Desktop: Confirm local storage
    
    opt Premium User - Background Sync
        Desktop->>Backend: POST /api/problems (metadata + image)
        Backend->>S3: Upload image
        S3-->>Backend: Return S3 key
        Backend->>PostgreSQL: Store problem metadata with S3 key
        PostgreSQL-->>Backend: Confirm cloud storage
    end
    
    Desktop-->>User: Show success confirmation
```

### 2. Problem Creation (Web - Manual Upload)
```mermaid
sequenceDiagram
    participant User
    participant Web
    participant IndexedDB
    participant Backend
    participant PostgreSQL
    participant S3
    
    User->>Web: Navigate to create problem
    User->>Web: Upload image file
    User->>Web: Fill problem details
    Web->>Backend: POST /api/problems (metadata + image)
    Backend->>S3: Upload image
    S3-->>Backend: Return S3 key
    Backend->>PostgreSQL: Store problem metadata with S3 key
    PostgreSQL-->>Backend: Confirm storage
    Backend-->>Web: Return problem data with signed URL
    Web->>IndexedDB: Cache problem metadata
    Web-->>User: Show success confirmation
```

### 3. Cloud Sync (Desktop - Premium Users)
```mermaid
sequenceDiagram
    participant Desktop
    participant Backend
    participant PostgreSQL
    participant S3
    participant SQLite
    participant FileSystem
    
    Note over Desktop: Bidirectional Sync Process
    
    %% PUSH: Upload local changes to cloud
    Desktop->>SQLite: Get problems created/updated since last sync
    SQLite-->>Desktop: Return local changes
    
    loop For each local change
        Desktop->>FileSystem: Read image file
        FileSystem-->>Desktop: Return image data
        Desktop->>Backend: POST /api/problems (metadata + image)
        Backend->>S3: Upload image
        S3-->>Backend: Return S3 key
        Backend->>PostgreSQL: Store/update metadata with S3 key
        PostgreSQL-->>Backend: Confirm storage
        Backend-->>Desktop: Confirm upload
        Desktop->>SQLite: Mark problem as synced
    end
    
    %% PULL: Download cloud changes to local
    Desktop->>SQLite: Get last sync timestamp
    SQLite-->>Desktop: Return timestamp
    Desktop->>Backend: GET /api/sync/problems?since=timestamp
    Backend->>PostgreSQL: Query updated problems from other devices
    PostgreSQL-->>Backend: Return cloud changes
    Backend->>S3: Generate signed URLs for new images
    S3-->>Backend: Return signed URLs
    Backend-->>Desktop: Return cloud changes + image URLs
    
    loop For each cloud change
        Desktop->>S3: Download image
        S3-->>Desktop: Return image data
        Desktop->>FileSystem: Save image locally
        Desktop->>SQLite: Store/update problem metadata
    end
    
    Desktop->>SQLite: Update last sync timestamp
```

### 4. Problem Retrieval and Display (Desktop)
```mermaid
sequenceDiagram
    participant User
    participant Desktop
    participant SQLite
    participant FileSystem
    
    User->>Desktop: Request to view problem
    Desktop->>SQLite: Query problem metadata
    SQLite-->>Desktop: Return metadata with file path
    Desktop->>FileSystem: Read image from local path
    FileSystem-->>Desktop: Return image data
    Desktop-->>User: Display problem with image
```

### 5. Problem Retrieval and Display (Web - Smart Caching)
```mermaid
sequenceDiagram
    participant User
    participant Web
    participant IndexedDB
    participant Backend
    participant PostgreSQL
    participant S3
    
    User->>Web: Request to view problem
    Web->>IndexedDB: Check cached problem
    
    alt Problem cached and fresh
        IndexedDB-->>Web: Return cached metadata + image URL
        Web-->>User: Display problem with image
    else Problem not cached or stale
        Web->>Backend: GET /api/problems/{id}
        Backend->>PostgreSQL: Query problem metadata
        PostgreSQL-->>Backend: Return metadata with S3 key
        Backend->>S3: Generate signed URL for image
        S3-->>Backend: Return signed URL
        Backend-->>Web: Return problem data + image URL
        Web->>IndexedDB: Cache problem for future use
        Web-->>User: Display problem with image
    end
```

### 6. Revision Mode (Desktop)
```mermaid
sequenceDiagram
    participant User
    participant Desktop
    participant SQLite
    participant FileSystem
    
    User->>Desktop: Enter revision mode
    User->>Desktop: Select scope (subject/course/folder)
    Desktop->>SQLite: Query problems with poor performance in scope
    SQLite-->>Desktop: Return filtered problem set
    Desktop->>FileSystem: Load images for problems
    FileSystem-->>Desktop: Return image data
    Desktop-->>User: Display problems for review
```

### 7. Revision Mode (Web)
```mermaid
sequenceDiagram
    participant User
    participant Web
    participant IndexedDB
    participant Backend
    participant PostgreSQL
    participant S3
    
    User->>Web: Enter revision mode
    User->>Web: Select scope (subject/course/folder)
    Web->>IndexedDB: Check for cached revision data
    
    alt Cached data available and fresh
        IndexedDB-->>Web: Return cached problem set
        Web-->>User: Display problems for review
    else No cache or stale data
        Web->>Backend: GET /api/problems/revision?scope=subject&performance=low
        Backend->>PostgreSQL: Query problems with poor performance
        PostgreSQL-->>Backend: Return filtered problem set
        Backend->>S3: Generate signed URLs for images
        S3-->>Backend: Return signed URLs
        Backend-->>Web: Return problems + image URLs
        Web->>IndexedDB: Cache revision data
        Web-->>User: Display problems for review
    end
```

### 8. User Authentication (Login)
```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant Backend
    participant PostgreSQL
    participant Stripe
    
    User->>Frontend: Enter credentials
    Frontend->>Backend: POST /api/auth/login
    Backend->>PostgreSQL: Verify user credentials
    PostgreSQL-->>Backend: Return user data + subscription status
    
    opt Premium User
        Backend->>Stripe: Verify subscription status
        Stripe-->>Backend: Return subscription details
    end
    
    Backend-->>Frontend: Return JWT token + user profile
    Frontend->>Frontend: Store auth token
    Frontend-->>User: Redirect to dashboard
    
    Note over Frontend: Enable premium features if applicable
```

### 9. Problem Editing
```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant SQLite
    participant FileSystem
    participant Backend
    participant PostgreSQL
    participant S3
    
    User->>Frontend: Edit problem (metadata or image)
    Frontend->>SQLite: Update problem metadata locally
    SQLite-->>Frontend: Confirm local update
    
    opt Image replacement
        Frontend->>FileSystem: Replace local image file
        FileSystem-->>Frontend: Confirm image update
    end
    
    Frontend->>SQLite: Mark problem as modified for sync
    Frontend-->>User: Show update confirmation
    
    opt Premium User - Background Sync
        Note over Frontend: Next sync will push changes to cloud
        Frontend->>Backend: PUT /api/problems/{id}
        
        opt Image was changed
            Backend->>S3: Upload new image
            S3-->>Backend: Return new S3 key
        end
        
        Backend->>PostgreSQL: Update problem metadata
        PostgreSQL-->>Backend: Confirm update
        Backend-->>Frontend: Confirm cloud sync
        Frontend->>SQLite: Mark as synced
    end
```

### 10. Sync Conflict Resolution
```mermaid
sequenceDiagram
    participant Desktop
    participant Backend
    participant PostgreSQL
    participant SQLite
    participant User
    
    Note over Desktop: During sync, conflict detected
    Desktop->>Backend: GET /api/sync/problems
    Backend->>PostgreSQL: Query problems with timestamps
    PostgreSQL-->>Backend: Return problem data
    Backend-->>Desktop: Return problems with last_modified timestamps
    
    Desktop->>SQLite: Compare timestamps with local data
    SQLite-->>Desktop: Return conflicts (same problem, different timestamps)
    
    alt Auto-resolve (last-write-wins)
        Desktop->>Desktop: Keep most recent version
        Desktop->>SQLite: Update local version
        Note over Desktop: Conflict resolved automatically
    else Manual resolution required
        Desktop->>User: Show conflict resolution dialog
        User->>Desktop: Choose version (local/remote/merge)
        
        alt Keep local version
            Desktop->>Backend: PUT /api/problems/{id} (force local version)
        else Keep remote version
            Desktop->>SQLite: Update to remote version
            Desktop->>FileSystem: Download remote image
        else Create duplicate
            Desktop->>SQLite: Save both versions with different IDs
        end
        
        Desktop-->>User: Show resolution confirmation
    end
```
