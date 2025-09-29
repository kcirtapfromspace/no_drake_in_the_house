#!/bin/bash

echo "🎵 Testing DNP List Endpoints"
echo "============================="

# Test getting DNP list
echo "1. Testing Get DNP List..."
GET_DNP_RESPONSE=$(curl -s http://localhost:3000/api/dnp)
echo "DNP List Response:"
echo "$GET_DNP_RESPONSE" | jq .
echo ""

# Test adding artist to DNP list
echo "2. Testing Add Artist to DNP List..."
ADD_DNP_RESPONSE=$(curl -s -X POST http://localhost:3000/api/dnp \
  -H "Content-Type: application/json" \
  -d '{
    "artist_query": "Drake",
    "provider": "spotify",
    "tags": ["hip-hop", "rap"],
    "note": "Personal preference - not my style"
  }')

echo "Add to DNP Response:"
echo "$ADD_DNP_RESPONSE" | jq .
echo ""

# Test artist search
echo "3. Testing Artist Search..."
SEARCH_RESPONSE=$(curl -s "http://localhost:3000/api/artists/search?q=drake")
echo "Search Response:"
echo "$SEARCH_RESPONSE" | jq .
echo ""

# Test removing from DNP list
echo "4. Testing Remove from DNP List..."
REMOVE_RESPONSE=$(curl -s -X DELETE http://localhost:3000/api/dnp/artist_123)
echo "Remove Response:"
echo "$REMOVE_RESPONSE" | jq .
echo ""

echo "✅ DNP endpoint testing complete!"
echo ""
echo "🎯 Next Steps:"
echo "   - Add real database integration"
echo "   - Add authentication middleware"
echo "   - Connect to Spotify/Apple Music APIs"