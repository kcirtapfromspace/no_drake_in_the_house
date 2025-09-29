#!/bin/bash

echo "🧪 Testing Local Authentication Endpoints"
echo "========================================"

# Test registration
echo "1. Testing Registration..."
REGISTER_RESPONSE=$(curl -s -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@musicblocklist.com",
    "password": "SecurePassword123!",
    "confirm_password": "SecurePassword123!"
  }')

echo "Registration Response:"
echo "$REGISTER_RESPONSE" | jq .
echo ""

# Test login
echo "2. Testing Login..."
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@musicblocklist.com",
    "password": "SecurePassword123!"
  }')

echo "Login Response:"
echo "$LOGIN_RESPONSE" | jq .

# Extract token for profile test
ACCESS_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.data.access_token // empty')
echo ""

if [ -n "$ACCESS_TOKEN" ] && [ "$ACCESS_TOKEN" != "null" ]; then
  echo "3. Testing Profile (with token)..."
  PROFILE_RESPONSE=$(curl -s -X GET http://localhost:3000/auth/profile \
    -H "Authorization: Bearer $ACCESS_TOKEN")
  
  echo "Profile Response:"
  echo "$PROFILE_RESPONSE" | jq .
  echo ""
  
  echo "4. Testing Logout..."
  LOGOUT_RESPONSE=$(curl -s -X POST http://localhost:3000/auth/logout \
    -H "Authorization: Bearer $ACCESS_TOKEN")
  
  echo "Logout Response:"
  echo "$LOGOUT_RESPONSE" | jq .
else
  echo "❌ No access token received, skipping profile and logout tests"
fi

echo ""
echo "✅ Local authentication endpoint testing complete!"