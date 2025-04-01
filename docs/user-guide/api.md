# API Reference

Muxly provides a comprehensive REST API for interacting with all components of the system. This guide provides an overview of the API endpoints and how to use them.

## API Overview

The Muxly API is:
- **RESTful**: Following REST principles for resource-based URLs, appropriate HTTP methods, and status codes
- **JSON-based**: All requests and responses use JSON format
- **Authenticated**: Most endpoints require authentication via JWT tokens
- **Versioned**: API versioning is done through URL prefixes (e.g., `/api/v1/`)
- **Documented**: Interactive documentation is available via Swagger UI

## Authentication

Most API endpoints require authentication. See the [Authentication Guide](authentication.md) for details on how to obtain and use authentication tokens.

Add the `Authorization` header to your requests:

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## API Documentation (Swagger UI)

Muxly includes an interactive API documentation interface using Swagger UI, which allows you to:
- Browse all available endpoints
- See request and response schemas
- Try out API calls directly from the browser

To access the Swagger UI:

1. Start your Muxly instance
2. Navigate to `http://your-muxly-host:3000/api/docs` in your web browser
3. If authentication is enabled, you'll need to authenticate first

## API Endpoints

### Core API Groups

The Muxly API is organized into these main groups:

1. **Authentication API**
   - `/api/auth/login` - Authenticate and get tokens
   - `/api/auth/refresh` - Refresh access token
   - `/api/auth/logout` - End the current session
   - `/api/auth/me` - Get current user information

2. **Connectors API**
   - `/api/connectors` - CRUD operations for data connectors
   - `/api/connectors/{id}/test` - Test a connector's connection
   - `/api/connectors/{id}/sync` - Manually trigger a data sync

3. **Router API**
   - `/api/routes` - CRUD operations for routes
   - `/api/destinations` - CRUD operations for destinations
   - `/api/routing-rules` - CRUD operations for routing rules

4. **Scheduler API**
   - `/api/jobs` - CRUD operations for scheduled jobs
   - `/api/jobs/{id}/trigger` - Manually trigger a job
   - `/api/jobs/{id}/history` - View job execution history

5. **User Management API**
   - `/api/users` - CRUD operations for users
   - `/api/roles` - CRUD operations for roles
   - `/api/permissions` - View available permissions

### Common Patterns

All resource endpoints follow these common patterns:

- `GET /api/{resource}` - List all resources
- `GET /api/{resource}/{id}` - Get a specific resource
- `POST /api/{resource}` - Create a new resource
- `PUT /api/{resource}/{id}` - Update a resource
- `DELETE /api/{resource}/{id}` - Delete a resource

### Filtering and Pagination

List endpoints support filtering and pagination:

```
GET /api/connectors?enabled=true&limit=10&offset=0
```

Common query parameters:
- `limit` - Maximum number of items to return
- `offset` - Number of items to skip
- `sort` - Field to sort by
- `order` - Sort order (`asc` or `desc`)

### Error Handling

The API uses standard HTTP status codes:

- `200 OK` - Request succeeded
- `201 Created` - Resource created
- `400 Bad Request` - Invalid request parameters
- `401 Unauthorized` - Authentication required
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error

Error responses follow a standard format:

```json
{
  "error": {
    "code": "invalid_request",
    "message": "The connector ID is invalid",
    "details": {
      "field": "connector_id",
      "reason": "must be a valid UUID"
    }
  }
}
```

## API Client Examples

### cURL

```bash
# Authenticate
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password123"}'

# Get connectors (with authentication)
curl -X GET http://localhost:3000/api/connectors \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### JavaScript (Fetch API)

```javascript
// Authenticate
async function login() {
  const response = await fetch('http://localhost:3000/api/auth/login', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      username: 'admin',
      password: 'password123'
    })
  });
  
  const data = await response.json();
  return data.access_token;
}

// Get connectors
async function getConnectors(token) {
  const response = await fetch('http://localhost:3000/api/connectors', {
    headers: {
      'Authorization': `Bearer ${token}`
    }
  });
  
  return await response.json();
}
```

### Python (Requests)

```python
import requests

# Authenticate
def login():
    response = requests.post(
        'http://localhost:3000/api/auth/login',
        json={'username': 'admin', 'password': 'password123'}
    )
    data = response.json()
    return data['access_token']

# Get connectors
def get_connectors(token):
    response = requests.get(
        'http://localhost:3000/api/connectors',
        headers={'Authorization': f'Bearer {token}'}
    )
    return response.json()
```

## Rate Limiting

The API implements rate limiting to prevent abuse:

- 100 requests per minute for authenticated users
- 10 requests per minute for unauthenticated users

Rate limit headers are included in all responses:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 99
X-RateLimit-Reset: 1617293999
```

If you exceed the rate limit, you'll receive a `429 Too Many Requests` response.

## API Versioning

The API is versioned to ensure backward compatibility. The current version is accessible at `/api/v1/`, but for simplicity, `/api/` also points to the current version.

When breaking changes are introduced, a new version will be available at `/api/v2/`, etc.

## Webhooks

Muxly can send webhook notifications for various events. See the [Webhook Configuration](webhooks.md) guide for details on how to set up and use webhooks. 