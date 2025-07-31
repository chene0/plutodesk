#!/bin/bash

echo "🔧 PlutoDesk Setup Validation"
echo "=============================="

# Check if backend is running
echo "🔍 Checking Backend..."
BACKEND_STATUS=$(curl -s http://localhost:8080/api/health 2>/dev/null || echo "failed")
if [[ $BACKEND_STATUS != "failed" ]]; then
    echo "✅ Backend is running: $BACKEND_STATUS"
    
    HELLO_RESPONSE=$(curl -s http://localhost:8080/api/hello 2>/dev/null)
    echo "✅ Test endpoint: $HELLO_RESPONSE"
else
    echo "❌ Backend is not running on port 8080"
    echo "💡 Start it with: ./start-backend.sh"
    exit 1
fi

# Check if frontend is running
echo ""
echo "🔍 Checking Frontend..."
FRONTEND_STATUS=$(curl -s -w "%{http_code}" http://localhost:3000 -o /dev/null 2>/dev/null || echo "failed")
if [[ $FRONTEND_STATUS == "200" ]]; then
    echo "✅ Frontend is running on http://localhost:3000"
else
    echo "❌ Frontend is not running on port 3000"
    echo "💡 Start it with: ./start-frontend.sh"
    exit 1
fi

echo ""
echo "🎉 Setup Validation Complete!"
echo ""
echo "🔗 Available Services:"
echo "   • Backend API: http://localhost:8080"
echo "   • Frontend Web: http://localhost:3000"
echo "   • API Health: http://localhost:8080/api/health"
echo "   • API Test: http://localhost:8080/api/hello"
echo ""
echo "🚀 Next Steps:"
echo "   • Open http://localhost:3000 in your browser"
echo "   • Run './start-tauri.sh' to test the desktop app"
echo "   • Both services support hot reload for development"
