#!/bin/bash

echo "Starting PlutoDesk Backend..."
cd backend

# Make gradlew executable if it isn't already
chmod +x ./gradlew

# Start the Spring Boot application
./gradlew bootRun
