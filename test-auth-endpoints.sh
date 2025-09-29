#!/bin/bash

echo "🧪 Testing Authentication Endpoints"
echo "=================================="

# Start port forward
kubectl port-forward svc/music-blocklist-manager-api 3000:80 -n music-blocklist-manager-dev > /dev/null 2>&1 &
PF_PID=$!

sleep 3

echo "1. Testing Registration..."
REGISTER_RESULT=$(curl -s -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@musicblocklist.com","password":"testpass123"}')

echo "Registration Response:"
echo "$REGISTER_RESULT" | jq .

echo ""
echo "2. Testing Login..."
LOGIN_RESULT=$(curl -s -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@musicblocklist.com","password":"testpass123"}')

echo "Login Response:"
echo "$LOGIN_RESULT" | jq .

# Extract token for further tests
TOKEN=$(echo "$LOGIN_RESULT" | jq -r '.data.access_token')

echo ""
echo "3. Testing Profile (with token)..."
PROFILE_RESULT=$(curl -s http://localhost:3000/auth/profile \
  -H "Authorization: Bearer $TOKEN")

echo "Profile Response:"
echo "$PROFILE_RESULT" | jq .

echo ""
echo "4. Testing Logout..."
LOGOUT_RESULT=$(curl -s -X POST http://localhost:3000/auth/logout \
  -H "Authorization: Bearer $TOKEN")

echo "Logout Response:"
echo "$LOGOUT_RESULT" | jq .

echo ""
echo "5. Testing API Status..."
STATUS_RESULT=$(curl -s http://localhost:3000/api/status)

echo "Status Response:"
echo "$STATUS_RESULT" | jq .

# Clean up
kill $PF_PID 2>/dev/null

echo ""
echo "✅ Authentication endpoint testing complete!"
echo ""
echo "🌐 To test the frontend:"
echo "   kubectl port-forward svc/music-blocklist-manager-frontend 8080:80 -n music-blocklist-manager-dev"
echo "   Then visit: http://localhost:8080"