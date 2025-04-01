# Authentication Guide

Muxly provides robust authentication capabilities including both local authentication and integration with external identity providers like Keycloak. This guide explains how to configure and use authentication in Muxly.

## Overview

Muxly's authentication system provides:

- Local username/password authentication
- Keycloak integration for SSO
- Role-based access control (RBAC)
- Permission-based authorization
- Secure token management
- Session tracking

## Configuration

### Local Authentication

To configure local authentication, set up the authentication section in your `config.yaml`:

```yaml
auth:
  provider: "local"
  session_timeout_minutes: 60
  token_secret: "${TOKEN_SECRET}"  # Set this in environment variable
  admin_user:
    username: "admin"
    password: "${ADMIN_PASSWORD}"  # Set this in environment variable
    email: "admin@example.com"
```

### Keycloak Integration

To use Keycloak for authentication:

```yaml
auth:
  provider: "keycloak"
  session_timeout_minutes: 60
  keycloak:
    server_url: "https://keycloak.example.com/auth"
    realm: "muxly"
    client_id: "muxly-app"
    client_secret: "${KEYCLOAK_CLIENT_SECRET}"  # Set this in environment variable
    admin_client_id: "admin-cli"
    admin_client_secret: "${KEYCLOAK_ADMIN_SECRET}"  # Set this in environment variable
```

### Environment Variables

Instead of storing sensitive values in the configuration file, you can use environment variables:

```bash
# For local authentication
export TOKEN_SECRET="your-secure-token-secret"
export ADMIN_PASSWORD="secure-admin-password"

# For Keycloak
export KEYCLOAK_CLIENT_SECRET="keycloak-client-secret"
export KEYCLOAK_ADMIN_SECRET="keycloak-admin-secret"
```

## Authentication API Endpoints

### Login

```
POST /api/auth/login
```

Request body:
```json
{
  "username": "user@example.com",
  "password": "password123"
}
```

Response:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### Refresh Token

```
POST /api/auth/refresh
```

Request body:
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

Response:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### Logout

```
POST /api/auth/logout
```

Headers:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

Response:
```json
{
  "message": "Logged out successfully"
}
```

### Get Current User

```
GET /api/auth/me
```

Headers:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

Response:
```json
{
  "id": "user123",
  "username": "user@example.com",
  "email": "user@example.com",
  "display_name": "John Doe",
  "roles": ["user"],
  "permissions": ["read:all", "manage:connectors"]
}
```

## User Management API Endpoints

### Create User

```
POST /api/users
```

Headers:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

Request body:
```json
{
  "username": "newuser",
  "email": "newuser@example.com",
  "display_name": "New User",
  "password": "secure-password",
  "roles": ["user"]
}
```

Response:
```json
{
  "id": "user456",
  "username": "newuser",
  "email": "newuser@example.com",
  "display_name": "New User",
  "roles": ["user"],
  "created_at": "2023-04-01T12:00:00Z"
}
```

### Get Users

```
GET /api/users
```

Headers:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

Response:
```json
{
  "users": [
    {
      "id": "user123",
      "username": "admin",
      "email": "admin@example.com",
      "display_name": "Administrator",
      "roles": ["admin"],
      "created_at": "2023-04-01T10:00:00Z"
    },
    {
      "id": "user456",
      "username": "newuser",
      "email": "newuser@example.com",
      "display_name": "New User",
      "roles": ["user"],
      "created_at": "2023-04-01T12:00:00Z"
    }
  ]
}
```

### Get User by ID

```
GET /api/users/{id}
```

Headers:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

Response:
```json
{
  "id": "user456",
  "username": "newuser",
  "email": "newuser@example.com",
  "display_name": "New User",
  "roles": ["user"],
  "permissions": ["read:all", "manage:connectors"],
  "created_at": "2023-04-01T12:00:00Z",
  "updated_at": "2023-04-01T12:00:00Z"
}
```

### Update User

```
PUT /api/users/{id}
```

Headers:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

Request body:
```json
{
  "display_name": "Updated User Name",
  "roles": ["user", "readonly"]
}
```

Response:
```json
{
  "id": "user456",
  "username": "newuser",
  "email": "newuser@example.com",
  "display_name": "Updated User Name",
  "roles": ["user", "readonly"],
  "updated_at": "2023-04-01T14:00:00Z"
}
```

### Delete User

```
DELETE /api/users/{id}
```

Headers:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

Response:
```json
{
  "message": "User deleted successfully"
}
```

## Role and Permission Management

### Get Roles

```
GET /api/roles
```

Headers:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

Response:
```json
{
  "roles": [
    {
      "id": "role_admin",
      "name": "admin",
      "description": "Administrator with full access"
    },
    {
      "id": "role_user",
      "name": "user",
      "description": "Regular user with limited access"
    },
    {
      "id": "role_readonly",
      "name": "readonly",
      "description": "Read-only access to the system"
    }
  ]
}
```

### Get Permissions

```
GET /api/permissions
```

Headers:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

Response:
```json
{
  "permissions": [
    {
      "id": "perm_admin_all",
      "name": "admin:all",
      "description": "Full administrative access",
      "resource": "*",
      "action": "*"
    },
    {
      "id": "perm_read_all",
      "name": "read:all",
      "description": "Read access to all resources",
      "resource": "*",
      "action": "read"
    },
    {
      "id": "perm_manage_connectors",
      "name": "manage:connectors",
      "description": "Manage connectors",
      "resource": "connectors",
      "action": "*"
    }
  ]
}
```

## Authentication in API Requests

All authenticated endpoints require a valid JWT token in the `Authorization` header:

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

You can obtain this token by calling the login endpoint, and you should include it in all subsequent requests.

## Troubleshooting

### Common Issues

1. **Invalid Credentials**: Ensure your username and password are correct
2. **Token Expired**: If you receive a 401 Unauthorized error, your token may have expired. Use the refresh token endpoint to get a new token
3. **Insufficient Permissions**: If you receive a 403 Forbidden error, your user account doesn't have the required permissions for the action

### Keycloak Integration Issues

1. **Unable to Connect to Keycloak**: Verify the server URL and realm are correct
2. **Invalid Client**: Check that the client ID and secret are correct
3. **User Not Found**: Ensure the user exists in the Keycloak realm

For detailed logs on authentication issues, you can increase the log level by setting:

```
RUST_LOG=debug
```

or

```
RUST_LOG=muxly::auth=trace
``` 