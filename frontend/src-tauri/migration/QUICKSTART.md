# Migration Quick Start Guide

## Setup

1. The `.env` file is already configured to use a local development database:
   ```env
   DATABASE_URL=sqlite://dev.db?mode=rwc
   ```

2. To use a different database, edit `.env` or copy from `.env.example`

## Common Commands

### Seed the database with test data
```bash
cargo run seed
```

This creates:
- 1 test user (test@example.com)
- 1 folder (Computer Science)
- 1 course (Data Structures & Algorithms)
- 1 subject (Binary Trees)
- 3 problems (Inorder Traversal, Find Height, LCA)
- 1 problem attempt

### Run migrations
```bash
cargo run
```

### Create a new migration
```bash
sea-orm-cli migrate generate <migration_name>
```

## Database Locations

**Development** (default in `.env`):
```
frontend/src-tauri/migration/dev.db
```

**Production** (where the app actually stores data):
- Windows: `%APPDATA%\com.plutodesk.app\plutodesk.db`
- macOS: `~/Library/Application Support/com.plutodesk.app/plutodesk.db`
- Linux: `~/.local/share/com.plutodesk.app/plutodesk.db`

## Tips

- **Re-seed the database**: Just run `cargo run seed` again - it automatically clears and re-seeds all data
- **Inspect data**: Use `sqlite3 dev.db` or any SQLite browser
- **Test on production DB**: Update `.env` to point to the actual app database path
- **Override for one command**: `DATABASE_URL="sqlite::memory:" cargo run seed`
- **Fresh start**: Delete `dev.db` entirely if you want to start completely fresh

## Querying Your Data

```bash
# Connect to the dev database
sqlite3 dev.db

# Useful queries:
SELECT * FROM users;
SELECT * FROM folders;
SELECT * FROM courses;
SELECT * FROM subjects;
SELECT * FROM problems;
SELECT * FROM problem_attempts;

# Check foreign key relationships
.schema folders
```
