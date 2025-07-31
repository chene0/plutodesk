#!/bin/bash

echo "Starting PlutoDesk Tauri App..."
cd frontend

# Install dependencies if needed
npm install

# Start Tauri development app
npm run tauri dev
