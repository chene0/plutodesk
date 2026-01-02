# PlutoDesk

A desktop application built with Next.js frontend, Spring Boot backend, and Tauri for desktop deployment.

## Architecture

- **Backend**: Spring Boot (Kotlin) running on port 8080
- **Frontend**: Next.js (React + TypeScript) running on port 3000
- **Desktop App**: Tauri wrapper around the Next.js frontend

## Prerequisites

- Java 21+
- Node.js 24+
- Rust (for Tauri)

## Quick Start

### Option 1: Start All Services (Recommended for development)
```bash
./start-dev.sh
```

This will start both the backend and frontend. Access:
- Frontend: http://localhost:3000
- Backend API: http://localhost:8080
- Health check: http://localhost:8080/api/health

### Option 2: Start Services Individually

#### Backend Only
```bash
./start-backend.sh
# OR manually:
cd backend && ./gradlew bootRun
```

#### Frontend Only (Web)
```bash
./start-frontend.sh
# OR manually:
cd frontend && npm install && npm run dev
```

#### Desktop App (Tauri)
```bash
./start-tauri.sh
# OR manually:
cd frontend && npm run tauri:dev
```

## Development Workflow

1. Start the development environment: `./start-dev.sh`
2. Make changes to your code
3. Both frontend and backend support hot reload
4. Test the desktop app with: `./start-tauri.sh`

## API Endpoints

- `GET /api/health` - Health check
- `GET /api/hello` - Test endpoint

## CORS Configuration

The backend is configured to accept requests from:
- http://localhost:3000 (Next.js dev server)
- tauri://localhost (Tauri app)
- http://localhost:1420 (Tauri dev server)

## Building for Production

### Backend
```bash
cd backend && ./gradlew build
```

### Frontend (Web)
```bash
cd frontend && npm run build
```

### Desktop App
```bash
cd frontend && npm run tauri:build
```

## Testing

### Unit Tests

#### Frontend Unit Tests
```bash
cd frontend && npm test
```

#### Backend Unit Tests
```bash
cd backend && ./gradlew test
```

#### Rust/Tauri Unit Tests
```bash
cd frontend/src-tauri && cargo test --lib
```

### End-to-End (E2E) Tests

E2E tests are located in the `e2e/` directory at the project root and use Playwright with Tauri support.

#### Prerequisites for E2E Testing
- All dependencies installed (see Prerequisites above)
- Frontend dependencies: `cd frontend && npm install`
- E2E dependencies: `cd e2e && npm install`

#### Running E2E Tests

1. **Install E2E dependencies** (first time only):
   ```bash
   cd e2e && npm install
   ```

2. **Run all E2E tests**:
   ```bash
   cd e2e && npm test
   ```
   This will automatically launch the Tauri app and run the tests.

3. **Run E2E tests with UI mode** (interactive):
   ```bash
   cd e2e && npm run test:ui
   ```

4. **Run E2E tests in debug mode**:
   ```bash
   cd e2e && npm run test:debug
   ```

5. **Run E2E tests in headed mode** (see browser):
   ```bash
   cd e2e && npm run test:headed
   ```

#### E2E Test Structure
- `e2e/tauri/` - Tauri desktop app E2E tests
- `e2e/api/` - Backend API E2E tests (future)
- `e2e/full-stack/` - Full stack integration tests (future)

**Note**: Currently, E2E tests contain skeleton test scenarios. Full implementation will be added as features are developed.

## Technology Stack

- **Backend**: Spring Boot 3.5, Kotlin, H2 Database
- **Frontend**: Next.js 15, React 19, TypeScript, Tailwind CSS
- **Desktop**: Tauri 2.5
- **Build Tools**: Gradle (backend), npm (frontend)
