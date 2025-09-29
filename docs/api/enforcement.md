# Enforcement API

## Overview

The Enforcement API handles the planning and execution of DNP list enforcement across connected streaming platforms. It provides dry-run capabilities, batch processing, and detailed progress tracking.

## Enforcement Planning

### POST /v1/enforcement/plan

Generate enforcement plan for connected platforms.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "providers": ["spotify", "apple"],  // Optional: specific providers
  "options": {
    "aggressiveness": "moderate",     // "conservative", "moderate", "aggressive"
    "block_collaborations": true,
    "block_featuring": true,
    "block_songwriter_only": false,
    "include_playlists": true,
    "include_library": true,
    "include_following": true
  },
  "dry_run": true,
  "community_lists": ["list_123", "list_456"]  // Optional: include community lists
}
```

**Response (200 OK):**
```json
{
  "plan_id": "plan_123456",
  "idempotency_key": "idem_789012",
  "dry_run": true,
  "created_at": "2024-01-15T10:30:00Z",
  "impact_summary": {
    "total_actions": 127,
    "estimated_duration": "180s",
    "platforms": {
      "spotify": {
        "capabilities": {
          "LIBRARY_PURGE": "SUPPORTED",
          "PLAYLIST_SCRUB": "SUPPORTED",
          "ARTIST_BLOCK": "SUPPORTED",
          "FEATURED_ARTIST_DETECTION": "SUPPORTED"
        },
        "actions": {
          "liked_songs": {
            "to_remove": 45,
            "collaborations_found": 12,
            "featuring_found": 8
          },
          "saved_albums": {
            "to_remove": 23,
            "collaborations_found": 5
          },
          "playlists": {
            "to_scrub": 12,
            "tracks_to_remove": 67,
            "collaborative_playlists": 3
          },
          "following": {
            "to_unfollow": 3
          }
        }
      },
      "apple": {
        "capabilities": {
          "LIBRARY_PURGE": "SUPPORTED",
          "PLAYLIST_SCRUB": "LIMITED",
          "ARTIST_BLOCK": "SUPPORTED"
        },
        "actions": {
          "library_songs": {
            "to_remove": 32,
            "featuring_found": 6
          },
          "playlists": {
            "to_scrub": 8,
            "tracks_to_remove": 28,
            "manual_review_required": 2
          }
        }
      }
    }
  },
  "warnings": [
    {
      "type": "LIMITED_CAPABILITY",
      "platform": "apple",
      "message": "Collaborative playlist modification not supported",
      "affected_items": 2
    }
  ],
  "resumable": true
}
```

### GET /v1/enforcement/plan/{plan_id}

Get enforcement plan details.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "plan_id": "plan_123456",
  "status": "ready",
  "created_at": "2024-01-15T10:30:00Z",
  "expires_at": "2024-01-15T11:30:00Z",
  "impact_summary": {
    // Same as plan creation response
  },
  "detailed_actions": [
    {
      "id": "action_001",
      "platform": "spotify",
      "type": "remove_liked_song",
      "entity": {
        "track_id": "4uLU6hMCjMI75M1A2tKUQC",
        "track_name": "Song Title",
        "artist_name": "Blocked Artist",
        "reason": "primary_artist"
      },
      "estimated_duration": "1s"
    },
    {
      "id": "action_002",
      "platform": "spotify",
      "type": "remove_playlist_track",
      "entity": {
        "playlist_id": "37i9dQZF1DX0XUsuxWHRQd",
        "playlist_name": "My Playlist",
        "track_id": "1A2B3C4D5E6F7G8H9I0J",
        "track_name": "Another Song",
        "artist_name": "Blocked Artist",
        "reason": "featuring_artist"
      },
      "estimated_duration": "2s"
    }
  ]
}
```

## Enforcement Execution

### POST /v1/enforcement/execute

Execute enforcement plan.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "plan_id": "plan_123456",
  "confirm_destructive": true,  // Required for non-dry-run executions
  "options": {
    "continue_on_error": true,
    "batch_size": 50,
    "delay_between_batches": "5s"
  }
}
```

**Response (202 Accepted):**
```json
{
  "execution_id": "exec_123456",
  "plan_id": "plan_123456",
  "status": "started",
  "started_at": "2024-01-15T10:35:00Z",
  "estimated_completion": "2024-01-15T10:38:00Z",
  "progress": {
    "total_actions": 127,
    "completed_actions": 0,
    "failed_actions": 0,
    "current_batch": 1,
    "total_batches": 3
  }
}
```

### GET /v1/enforcement/execute/{execution_id}

Get enforcement execution status.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "execution_id": "exec_123456",
  "plan_id": "plan_123456",
  "status": "in_progress",  // "started", "in_progress", "completed", "failed", "cancelled"
  "started_at": "2024-01-15T10:35:00Z",
  "updated_at": "2024-01-15T10:36:30Z",
  "progress": {
    "total_actions": 127,
    "completed_actions": 85,
    "failed_actions": 2,
    "current_batch": 2,
    "total_batches": 3,
    "percentage": 67
  },
  "current_operation": {
    "platform": "spotify",
    "operation": "removing_playlist_tracks",
    "playlist_name": "My Favorites"
  },
  "platform_status": {
    "spotify": {
      "status": "in_progress",
      "completed": 65,
      "failed": 1,
      "remaining": 20
    },
    "apple": {
      "status": "completed",
      "completed": 20,
      "failed": 1,
      "remaining": 0
    }
  }
}
```

### POST /v1/enforcement/execute/{execution_id}/cancel

Cancel ongoing enforcement execution.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "execution_id": "exec_123456",
  "status": "cancelling",
  "message": "Cancellation requested. Current batch will complete before stopping."
}
```

## Enforcement History

### GET /v1/enforcement/history

Get user's enforcement history.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Query Parameters:**
- `cursor` (optional): Pagination cursor
- `limit` (optional): Number of items per page (max 50, default 20)
- `status` (optional): Filter by status
- `platform` (optional): Filter by platform
- `from_date` (optional): Filter from date (ISO 8601)
- `to_date` (optional): Filter to date (ISO 8601)

**Response (200 OK):**
```json
{
  "data": [
    {
      "execution_id": "exec_123456",
      "plan_id": "plan_123456",
      "status": "completed",
      "started_at": "2024-01-15T10:35:00Z",
      "completed_at": "2024-01-15T10:37:45Z",
      "duration": "2m45s",
      "summary": {
        "total_actions": 127,
        "successful_actions": 125,
        "failed_actions": 2,
        "platforms": ["spotify", "apple"]
      },
      "rollback_available": true
    }
  ],
  "pagination": {
    "next_cursor": "eyJpZCI6IjEyMyJ9",
    "has_more": true,
    "total_count": 45
  }
}
```

### GET /v1/enforcement/history/{execution_id}

Get detailed enforcement execution results.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "execution_id": "exec_123456",
  "plan_id": "plan_123456",
  "status": "completed",
  "started_at": "2024-01-15T10:35:00Z",
  "completed_at": "2024-01-15T10:37:45Z",
  "summary": {
    "total_actions": 127,
    "successful_actions": 125,
    "failed_actions": 2
  },
  "detailed_results": [
    {
      "action_id": "action_001",
      "platform": "spotify",
      "type": "remove_liked_song",
      "status": "success",
      "entity": {
        "track_id": "4uLU6hMCjMI75M1A2tKUQC",
        "track_name": "Song Title",
        "artist_name": "Blocked Artist"
      },
      "executed_at": "2024-01-15T10:35:15Z",
      "duration": "1.2s"
    },
    {
      "action_id": "action_002",
      "platform": "spotify",
      "type": "remove_playlist_track",
      "status": "failed",
      "entity": {
        "playlist_id": "37i9dQZF1DX0XUsuxWHRQd",
        "track_id": "1A2B3C4D5E6F7G8H9I0J"
      },
      "error": {
        "code": "INSUFFICIENT_PERMISSIONS",
        "message": "Cannot modify collaborative playlist owned by another user"
      },
      "executed_at": "2024-01-15T10:35:30Z"
    }
  ],
  "rollback_info": {
    "available": true,
    "expires_at": "2024-01-22T10:37:45Z",
    "reversible_actions": 125,
    "irreversible_actions": 0
  }
}
```

## Rollback Operations

### POST /v1/enforcement/rollback

Rollback enforcement execution.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "execution_id": "exec_123456",
  "rollback_type": "full",  // "full", "partial", "selective"
  "action_ids": [],         // Required for "selective" rollback
  "confirm_rollback": true
}
```

**Response (202 Accepted):**
```json
{
  "rollback_id": "rollback_123456",
  "execution_id": "exec_123456",
  "status": "started",
  "rollback_type": "full",
  "started_at": "2024-01-15T11:00:00Z",
  "estimated_completion": "2024-01-15T11:02:00Z",
  "actions_to_rollback": 125
}
```

### GET /v1/enforcement/rollback/{rollback_id}

Get rollback operation status.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "rollback_id": "rollback_123456",
  "execution_id": "exec_123456",
  "status": "completed",
  "started_at": "2024-01-15T11:00:00Z",
  "completed_at": "2024-01-15T11:01:45Z",
  "summary": {
    "total_actions": 125,
    "successful_rollbacks": 123,
    "failed_rollbacks": 2,
    "irreversible_actions": 0
  }
}
```

## Batch Operations

### GET /v1/enforcement/batches

Get active enforcement batches.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "active_batches": [
    {
      "batch_id": "batch_123456",
      "execution_id": "exec_123456",
      "platform": "spotify",
      "status": "processing",
      "started_at": "2024-01-15T10:36:00Z",
      "progress": {
        "total_items": 50,
        "completed_items": 35,
        "failed_items": 1
      }
    }
  ],
  "queue_status": {
    "pending_batches": 2,
    "estimated_wait_time": "30s"
  }
}
```

## Error Responses

### 400 Bad Request - Invalid Plan
```json
{
  "error": {
    "code": "INVALID_ENFORCEMENT_PLAN",
    "message": "Enforcement plan has expired or is invalid",
    "details": {
      "plan_id": "plan_123456",
      "expires_at": "2024-01-15T11:30:00Z",
      "current_time": "2024-01-15T11:35:00Z"
    }
  }
}
```

### 409 Conflict - Execution In Progress
```json
{
  "error": {
    "code": "ENFORCEMENT_IN_PROGRESS",
    "message": "Another enforcement operation is already running",
    "details": {
      "active_execution_id": "exec_789012",
      "estimated_completion": "2024-01-15T10:40:00Z"
    }
  }
}
```

### 502 Bad Gateway - Platform Error
```json
{
  "error": {
    "code": "PLATFORM_API_ERROR",
    "message": "Spotify API is currently unavailable",
    "details": {
      "platform": "spotify",
      "error_code": "SERVICE_UNAVAILABLE",
      "retry_after": 300
    }
  }
}
```

### 503 Service Unavailable - Rate Limited
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Platform rate limit exceeded. Operation queued.",
    "details": {
      "platform": "spotify",
      "queue_position": 3,
      "estimated_start_time": "2024-01-15T10:45:00Z"
    }
  }
}
```