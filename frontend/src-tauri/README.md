# Plutodesk Tauri Backend

This directory contains the Rust backend for the Plutodesk Tauri application, including database management, migrations, and Tauri commands.

## Database Management

The application uses SQLite with SeaORM for database management.

### Running Migrations

Migrations are automatically run when the application starts. The database file will be created in the appropriate app data directory for your platform:
- **Windows**: `%APPDATA%\com.plutodesk.app\plutodesk.db`
- **macOS**: `~/Library/Application Support/com.plutodesk.app/plutodesk.db`
- **Linux**: `~/.local/share/com.plutodesk.app/plutodesk.db`

You can also run migrations manually:

```bash
cd migration
cargo run
```

### Seeding the Database

To seed the database with test data for development:

```bash
cd migration
cargo run seed
```

The seed command:
- Automatically reads the database URL from the `.env` file in the `migration` directory
- By default, creates a `dev.db` file in the migration directory
- **Is idempotent** - it clears all existing data before seeding, so you can run it multiple times

**Customizing the database location:**

Edit `migration/.env` to point to a different database:
```env
# Use the actual app database
DATABASE_URL=sqlite://C:/Users/YourUsername/AppData/Roaming/com.plutodesk.app/plutodesk.db?mode=rwc

# Or use in-memory (data won't persist)
DATABASE_URL=sqlite::memory:?mode=rwc
```

You can also override it via command line:
```bash
DATABASE_URL="sqlite://custom.db?mode=rwc" cargo run seed
```

**What gets seeded:**
- A test user
- A Computer Science folder
- A Data Structures & Algorithms course
- A Binary Trees subject
- 3 sample problems (Inorder Traversal, Find Height, Lowest Common Ancestor)
- A sample problem attempt

### Running Tests

The project includes comprehensive test coverage for database services. Tests use in-memory SQLite databases for isolation and speed.

Run all tests:

```bash
cargo test --lib
```

Run specific test modules:

```bash
# Test folder service
cargo test --lib db::services::folders::tests

# Test problem service
cargo test --lib db::services::problems::tests
```

Run with output:

```bash
cargo test --lib -- --nocapture
```

### Test Coverage

Current test coverage includes:

**Folders Service** (`src/db/services/folders.rs`):
- Creating folders
- Getting folders by ID and user
- Updating folder properties
- Deleting folders
- User isolation

**Problems Service** (`src/db/services/problems.rs`):
- Creating problems
- Updating problem metadata
- Success rate calculation (crucial for spaced repetition)
- Problem statistics tracking
- Getting problems by subject
- Deleting problems

Each test uses an isolated in-memory database with full migrations applied.

## Database Architecture

The database follows a hierarchical structure:

```
Users
└── Folders
    └── Courses
        └── Subjects
            └── Problems
                └── Problem Attempts
```

All relationships use foreign key constraints with CASCADE delete to maintain referential integrity.

## Available Tauri Commands

All CRUD operations are exposed as Tauri commands that can be invoked from the frontend:

### Folders
- `create_folder`
- `get_folder`
- `get_folders_by_user`
- `update_folder`
- `delete_folder`

### Courses
- `create_course`
- `get_course`
- `get_courses_by_folder`
- `update_course`
- `delete_course`

### Subjects
- `create_subject`
- `get_subject`
- `get_subjects_by_course`
- `update_subject`
- `delete_subject`

### Problems
- `create_problem`
- `get_problem`
- `get_problems_by_subject`
- `update_problem`
- `update_problem_stats`
- `delete_problem`

### Problem Attempts
- `create_problem_attempt`
- `get_problem_attempt`
- `get_attempts_by_problem`
- `update_problem_attempt`
- `delete_problem_attempt`

## Development

### Adding New Migrations

```bash
cd migration
sea-orm-cli migrate generate <migration_name>
```

### Project Structure

```
frontend/src-tauri/
├── src/
│   ├── commands/         # Tauri command handlers
│   ├── db/
│   │   ├── entities/     # SeaORM entity models
│   │   ├── services/     # Business logic layer (with tests)
│   │   ├── init.rs       # Database initialization
│   │   └── mod.rs
│   ├── screenshot.rs     # Screenshot functionality
│   └── lib.rs            # Main entry point
├── migration/
│   ├── src/
│   │   ├── seed.rs       # Database seeding
│   │   └── m*.rs         # Migration files
│   └── Cargo.toml
└── Cargo.toml
```

## Best Practices

1. **Keep commands thin** - Business logic belongs in the service layer, not in command handlers
2. **Test services, not commands** - Service functions are easier to test than Tauri commands
3. **Use transactions** - For operations that modify multiple tables
4. **Run tests before committing** - Ensure `cargo test --lib` passes
5. **Use the seed command** - For consistent test data across development environments
