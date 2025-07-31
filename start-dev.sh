#!/bin/bash

# Start all services for PlutoDesk development

echo "🚀 Starting PlutoDesk Development Environment"
echo "=============================================="

# Function to kill background processes on exit
cleanup() {
    echo "🛑 Stopping all services..."
    jobs -p | xargs kill 2>/dev/null
    exit 0
}

# Set up cleanup on script exit
trap cleanup SIGINT SIGTERM EXIT

# Start backend
echo "📦 Starting Backend (Spring Boot)..."
cd backend
chmod +x ./gradlew
./gradlew bootRun &
BACKEND_PID=$!

# Wait a moment for backend to start
sleep 3

# Start frontend
echo "🌐 Starting Frontend (Next.js)..."
cd ../frontend
npm install
npm run dev &
FRONTEND_PID=$!

echo ""
echo "✅ Services started!"
echo "🔗 Backend API: http://localhost:8080"
echo "🔗 Frontend: http://localhost:3000"
echo "🔗 Backend Health: http://localhost:8080/api/health"
echo "🔗 Backend Test: http://localhost:8080/api/hello"
echo ""
echo "💡 To start the Tauri desktop app, run: ./start-tauri.sh"
echo "🛑 Press Ctrl+C to stop all services"

# Wait for background processes
wait
