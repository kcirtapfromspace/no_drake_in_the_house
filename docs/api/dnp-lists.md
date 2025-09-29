# DNP Lists API

## Overview

The DNP (Do-Not-Play) Lists API allows users to manage their personal blocklists of artists they want to avoid across streaming platforms.

## Endpoints

### GET /v1/dnp/lists

Get user's DNP lists.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Query Parameters:**
- `cursor` (optional): Pagination cursor
- `limit` (optional): Number of items per page (max 100, default 20)
- `search` (optional): Search term for artist names
- `tags` (optional): Filter by tags (comma-separated)

**Response (200 OK):**
```json
{
  "data": [
    {
      "id": "dnp_123456",
      "artist": {
        "id": "artist_789",
        "canonical_name": "Example Artist",
        "external_ids": {
          "spotify": "4uLU6hMCjMI75M1A2tKUQC",
          "apple": "159260351",
          "musicbrainz": "f27ec8db-af05-4f36-916e-3d57f91ecf5e"
        },
        "metadata": {
          "image": "https://i.scdn.co/image/ab67616d0000b273...",
          "genres": ["hip-hop", "rap"]
        }
      },
      "tags": ["personal", "explicit"],
      "note": "Personal preference",
      "created_at": "2024-01-15T10:30:00Z"
    }
  ],
  "pagination": {
    "next_cursor": "eyJpZCI6IjEyMyJ9",
    "has_more": true,
    "total_count": 150
  }
}
```

### POST /v1/dnp/lists

Add artist to DNP list.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "artist_query": "Drake",  // Artist name or provider URL
  "provider": "spotify",   // Optional: preferred provider for search
  "tags": ["personal"],
  "note": "Personal preference"
}
```

**Response (201 Created):**
```json
{
  "id": "dnp_123456",
  "artist": {
    "id": "artist_789",
    "canonical_name": "Drake",
    "external_ids": {
      "spotify": "3TVXtAsR1Inumwj472S9r4",
      "apple": "271256",
      "musicbrainz": "2ee85c5a-871e-4e4d-8c7b-c5b5e6e5e5e5"
    },
    "metadata": {
      "image": "https://i.scdn.co/image/ab67616d0000b273...",
      "genres": ["hip-hop", "rap", "pop"]
    }
  },
  "tags": ["personal"],
  "note": "Personal preference",
  "created_at": "2024-01-15T10:30:00Z"
}
```

### PUT /v1/dnp/lists/{dnp_id}

Update DNP list entry.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "tags": ["personal", "updated"],
  "note": "Updated note"
}
```

**Response (200 OK):**
```json
{
  "id": "dnp_123456",
  "artist": {
    "id": "artist_789",
    "canonical_name": "Drake",
    "external_ids": {
      "spotify": "3TVXtAsR1Inumwj472S9r4",
      "apple": "271256"
    }
  },
  "tags": ["personal", "updated"],
  "note": "Updated note",
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T11:00:00Z"
}
```

### DELETE /v1/dnp/lists/{dnp_id}

Remove artist from DNP list.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (204 No Content)**

### POST /v1/dnp/lists/bulk

Bulk operations on DNP list.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "operation": "add",  // "add", "remove", "update_tags"
  "artists": [
    {
      "artist_query": "Artist Name 1",
      "tags": ["bulk-import"],
      "note": "Imported from CSV"
    },
    {
      "artist_query": "Artist Name 2",
      "tags": ["bulk-import"]
    }
  ]
}
```

**Response (200 OK):**
```json
{
  "batch_id": "batch_123456",
  "status": "processing",
  "summary": {
    "total_requested": 2,
    "successful": 0,
    "failed": 0,
    "processing": 2
  },
  "results": []
}
```

### GET /v1/dnp/lists/bulk/{batch_id}

Get bulk operation status.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "batch_id": "batch_123456",
  "status": "completed",
  "summary": {
    "total_requested": 2,
    "successful": 2,
    "failed": 0,
    "processing": 0
  },
  "results": [
    {
      "artist_query": "Artist Name 1",
      "status": "success",
      "dnp_entry": {
        "id": "dnp_789",
        "artist": {
          "canonical_name": "Artist Name 1"
        }
      }
    },
    {
      "artist_query": "Artist Name 2",
      "status": "success",
      "dnp_entry": {
        "id": "dnp_790",
        "artist": {
          "canonical_name": "Artist Name 2"
        }
      }
    }
  ]
}
```

## Artist Search

### GET /v1/artists/search

Search for artists across platforms.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Query Parameters:**
- `q` (required): Search query
- `provider` (optional): Preferred provider ("spotify", "apple", "youtube")
- `limit` (optional): Number of results (max 50, default 10)

**Response (200 OK):**
```json
{
  "data": [
    {
      "id": "artist_123",
      "canonical_name": "Drake",
      "external_ids": {
        "spotify": "3TVXtAsR1Inumwj472S9r4",
        "apple": "271256",
        "musicbrainz": "2ee85c5a-871e-4e4d-8c7b-c5b5e6e5e5e5"
      },
      "metadata": {
        "image": "https://i.scdn.co/image/ab67616d0000b273...",
        "genres": ["hip-hop", "rap", "pop"],
        "popularity": 95,
        "followers": 50000000
      },
      "provider_badges": ["spotify", "apple", "youtube"],
      "confidence_score": 0.98
    }
  ],
  "query": "Drake",
  "total_results": 1
}
```

## Import/Export

### POST /v1/dnp/lists/import

Import DNP list from file.

**Headers:**
```
Authorization: Bearer <access_token>
Content-Type: multipart/form-data
```

**Form Data:**
- `file`: CSV or JSON file
- `format`: "csv" or "json"
- `merge_strategy`: "replace" or "merge"

**CSV Format:**
```csv
artist_name,tags,note
"Drake","personal,hip-hop","Personal preference"
"Artist Name 2","imported","From old list"
```

**JSON Format:**
```json
{
  "artists": [
    {
      "artist_name": "Drake",
      "tags": ["personal", "hip-hop"],
      "note": "Personal preference"
    }
  ]
}
```

**Response (202 Accepted):**
```json
{
  "import_id": "import_123456",
  "status": "processing",
  "estimated_completion": "2024-01-15T10:35:00Z"
}
```

### GET /v1/dnp/lists/import/{import_id}

Get import status.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "import_id": "import_123456",
  "status": "completed",
  "summary": {
    "total_rows": 100,
    "successful": 95,
    "failed": 5,
    "skipped": 0
  },
  "errors": [
    {
      "row": 15,
      "artist_name": "Unknown Artist",
      "error": "Artist not found in any provider"
    }
  ],
  "completed_at": "2024-01-15T10:34:30Z"
}
```

### GET /v1/dnp/lists/export

Export DNP list.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Query Parameters:**
- `format`: "csv" or "json"
- `include_metadata`: "true" or "false" (default: false)

**Response (200 OK):**
```
Content-Type: application/json
Content-Disposition: attachment; filename="dnp-list-2024-01-15.json"

{
  "exported_at": "2024-01-15T10:30:00Z",
  "total_artists": 150,
  "artists": [
    {
      "canonical_name": "Drake",
      "external_ids": {
        "spotify": "3TVXtAsR1Inumwj472S9r4"
      },
      "tags": ["personal"],
      "note": "Personal preference",
      "added_at": "2024-01-10T15:20:00Z"
    }
  ]
}
```

## Tags Management

### GET /v1/dnp/tags

Get user's DNP list tags.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "tags": [
    {
      "name": "personal",
      "count": 45,
      "color": "#ff6b6b"
    },
    {
      "name": "explicit",
      "count": 23,
      "color": "#4ecdc4"
    }
  ]
}
```

### PUT /v1/dnp/tags/{tag_name}

Update tag properties.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "color": "#ff6b6b",
  "description": "Personal preferences"
}
```

**Response (200 OK):**
```json
{
  "name": "personal",
  "count": 45,
  "color": "#ff6b6b",
  "description": "Personal preferences"
}
```

## Error Responses

### 400 Bad Request - Artist Not Found
```json
{
  "error": {
    "code": "ARTIST_NOT_FOUND",
    "message": "Could not find artist matching query",
    "details": {
      "query": "Unknown Artist Name",
      "searched_providers": ["spotify", "apple", "musicbrainz"]
    }
  }
}
```

### 409 Conflict - Duplicate Entry
```json
{
  "error": {
    "code": "DUPLICATE_ENTRY",
    "message": "Artist is already in your DNP list",
    "details": {
      "existing_entry_id": "dnp_123456",
      "artist_name": "Drake"
    }
  }
}
```

### 413 Payload Too Large
```json
{
  "error": {
    "code": "IMPORT_FILE_TOO_LARGE",
    "message": "Import file exceeds maximum size limit",
    "details": {
      "max_size": "10MB",
      "received_size": "15MB"
    }
  }
}
```