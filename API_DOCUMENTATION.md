# DencapsBI API Documentation

Base URL: `http://localhost:8080` (development) or your production URL

## Authentication

All protected endpoints require a JWT token in the Authorization header:
```
Authorization: Bearer <access_token>
```

## Public Endpoints

### Register User
**POST** `/api/auth/register`

Creates a new user account.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "securepassword123",
  "name": "John Doe",
  "tenant_id": "org-123"
}
```

**Response:** (201 Created)
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "user": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "name": "John Doe",
    "role": "viewer",
    "tenant_id": "org-123",
    "is_active": true
  }
}
```

### Login
**POST** `/api/auth/login`

Authenticates a user and returns JWT tokens.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "securepassword123"
}
```

**Response:** (200 OK)
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "user": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "name": "John Doe",
    "role": "viewer",
    "tenant_id": "org-123",
    "is_active": true
  }
}
```

### Refresh Token
**POST** `/api/auth/refresh`

Refreshes the access token using a refresh token.

**Request Body:**
```json
{
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGc..."
}
```

**Response:** (200 OK)
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc..."
}
```

## Protected Endpoints

### Get Current User
**GET** `/api/users/me`

Returns the currently authenticated user's information.

**Response:** (200 OK)
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "name": "John Doe",
  "role": "viewer",
  "tenant_id": "org-123",
  "is_active": true
}
```

### Create Project
**POST** `/api/projects`

Creates a new project for the current user.

**Request Body:**
```json
{
  "name": "Sales Analytics",
  "description": "Q4 2024 sales data analysis"
}
```

**Response:** (201 Created)
```json
{
  "project_id": "660e8400-e29b-41d4-a716-446655440000",
  "name": "Sales Analytics",
  "description": "Q4 2024 sales data analysis",
  "tenant_id": "org-123",
  "owner_id": "550e8400-e29b-41d4-a716-446655440000",
  "is_active": true,
  "created_at": "2024-01-07T19:00:00Z"
}
```

### Get User Projects
**GET** `/api/projects`

Returns all projects accessible to the current user.

**Response:** (200 OK)
```json
[
  {
    "project_id": "660e8400-e29b-41d4-a716-446655440000",
    "name": "Sales Analytics",
    "description": "Q4 2024 sales data analysis",
    "tenant_id": "org-123",
    "owner_id": "550e8400-e29b-41d4-a716-446655440000",
    "is_active": true,
    "created_at": "2024-01-07T19:00:00Z"
  }
]
```

### Get Project by ID
**GET** `/api/projects/{project_id}`

Returns a specific project by ID if the user has access.

**Response:** (200 OK)
```json
{
  "project_id": "660e8400-e29b-41d4-a716-446655440000",
  "name": "Sales Analytics",
  "description": "Q4 2024 sales data analysis",
  "tenant_id": "org-123",
  "owner_id": "550e8400-e29b-41d4-a716-446655440000",
  "is_active": true,
  "created_at": "2024-01-07T19:00:00Z"
}
```

### Create Analytics Query
**POST** `/api/analytics/queries`

Creates a new analytics query for AI processing.

**Request Body:**
```json
{
  "project_id": "660e8400-e29b-41d4-a716-446655440000",
  "query_text": "What are the top 5 products by revenue in Q4?"
}
```

**Response:** (201 Created)
```json
{
  "query_id": "770e8400-e29b-41d4-a716-446655440000",
  "project_id": "660e8400-e29b-41d4-a716-446655440000",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "query_text": "What are the top 5 products by revenue in Q4?",
  "response_text": null,
  "status": "Pending",
  "created_at": "2024-01-07T19:05:00Z",
  "completed_at": null
}
```

### Process Query
**POST** `/api/analytics/queries/{query_id}/process`

Processes a query with the AI model.

**Response:** (200 OK)
```json
{
  "query_id": "770e8400-e29b-41d4-a716-446655440000",
  "response": "Based on the Q4 data, here are the top 5 products by revenue:\n1. Product A - $250,000\n2. Product B - $180,000\n3. Product C - $150,000\n4. Product D - $120,000\n5. Product E - $95,000\n\nProduct A showed significant growth..."
}
```

### Get Query by ID
**GET** `/api/analytics/queries/{query_id}`

Returns a specific query and its results.

**Response:** (200 OK)
```json
{
  "query_id": "770e8400-e29b-41d4-a716-446655440000",
  "project_id": "660e8400-e29b-41d4-a716-446655440000",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "query_text": "What are the top 5 products by revenue in Q4?",
  "response_text": "Based on the Q4 data...",
  "status": "Completed",
  "created_at": "2024-01-07T19:05:00Z",
  "completed_at": "2024-01-07T19:05:15Z"
}
```

### Get Project Queries
**GET** `/api/analytics/projects/{project_id}/queries`

Returns all queries for a specific project.

**Response:** (200 OK)
```json
[
  {
    "query_id": "770e8400-e29b-41d4-a716-446655440000",
    "project_id": "660e8400-e29b-41d4-a716-446655440000",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "query_text": "What are the top 5 products by revenue in Q4?",
    "response_text": "Based on the Q4 data...",
    "status": "Completed",
    "created_at": "2024-01-07T19:05:00Z",
    "completed_at": "2024-01-07T19:05:15Z"
  }
]
```

## Error Responses

All endpoints may return error responses in the following format:

**400 Bad Request:**
```json
{
  "error": "Validation error: email is required"
}
```

**401 Unauthorized:**
```json
{
  "error": "Invalid token"
}
```

**403 Forbidden:**
```json
{
  "error": "Access denied to this project"
}
```

**404 Not Found:**
```json
{
  "error": "Project not found"
}
```

**429 Too Many Requests:**
```json
{
  "error": "Rate limit exceeded"
}
```

**500 Internal Server Error:**
```json
{
  "error": "Database error: connection failed"
}
```

## Rate Limiting

- Default: 100 requests per 60 seconds per IP address
- Configurable via `RATE_LIMIT_REQUESTS` and `RATE_LIMIT_WINDOW_SECS` environment variables
- Applies to all protected endpoints

## User Roles

1. **Admin** - Full system access
2. **Project Owner** - Can create and manage projects, add members
3. **Project Member** - Can access assigned projects and create queries
4. **Viewer** - Read-only access to assigned projects

## Query Status

- **Pending** - Query created, awaiting processing
- **Processing** - Query is being processed by AI
- **Completed** - Query successfully processed
- **Failed** - Query processing failed
