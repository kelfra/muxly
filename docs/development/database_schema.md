# Database Schema Documentation

This document provides an overview of the database schema used in Muxly.

## Schema Overview

The Muxly database is organized into several related tables to support the core features:

1. **Core Tables**
   - Connectors
   - Settings

2. **Router Tables**
   - Destinations
   - Routing Rules
   - Rule Transformations
   - Routes

3. **Scheduler Tables**
   - Scheduler Jobs
   - Job Executions
   - Webhook Triggers

4. **Authentication Tables**
   - Users
   - Roles
   - Permissions
   - User Roles
   - Role Permissions
   - Auth Sessions

## Core Tables

### Connectors

```sql
CREATE TABLE IF NOT EXISTS connectors (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    connector_type TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    auth_settings TEXT NOT NULL,  -- JSON blob for authentication settings
    connection_settings TEXT NOT NULL,  -- JSON blob for connection settings
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

This table stores all configured connectors, including their authentication and connection settings.

### Settings

```sql
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

Stores global application settings in a key-value format.

## Router Tables

### Destinations

```sql
CREATE TABLE IF NOT EXISTS destinations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    destination_type TEXT NOT NULL,
    config TEXT NOT NULL, -- JSON blob for destination configuration
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

Stores all configured destinations where data can be sent.

### Routing Rules

```sql
CREATE TABLE IF NOT EXISTS routing_rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 100,
    enabled BOOLEAN NOT NULL DEFAULT true,
    condition TEXT, -- JSON blob for rule condition
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

Defines rules for routing data to destinations based on conditions.

### Rule Transformations

```sql
CREATE TABLE IF NOT EXISTS rule_transformations (
    id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL,
    transformation_type TEXT NOT NULL,
    config TEXT NOT NULL,
    sequence_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (rule_id) REFERENCES routing_rules(id) ON DELETE CASCADE
);
```

Defines transformations to apply to data as part of routing rules.

### Routes

```sql
CREATE TABLE IF NOT EXISTS routes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    connector_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    error_handling TEXT NOT NULL DEFAULT '{"mode": "continue"}',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (connector_id) REFERENCES connectors(id) ON DELETE CASCADE
);
```

Defines data routes from connectors to destinations through rules.

## Scheduler Tables

### Scheduler Jobs

```sql
CREATE TABLE IF NOT EXISTS scheduler_jobs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    job_type TEXT NOT NULL,
    schedule TEXT,
    connector_id TEXT NOT NULL,
    action_type TEXT NOT NULL,
    action_params TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    max_retries INTEGER NOT NULL DEFAULT 3,
    backoff_strategy TEXT NOT NULL DEFAULT 'exponential',
    timeout_seconds INTEGER NOT NULL DEFAULT 300,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (connector_id) REFERENCES connectors(id) ON DELETE CASCADE
);
```

Defines scheduled jobs for data synchronization or other actions.

### Job Executions

```sql
CREATE TABLE IF NOT EXISTS job_executions (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    status TEXT NOT NULL,
    result TEXT,
    attempt INTEGER NOT NULL DEFAULT 1,
    triggered_by TEXT NOT NULL DEFAULT 'schedule',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (job_id) REFERENCES scheduler_jobs(id) ON DELETE CASCADE
);
```

Records each execution of a scheduled job, including its status and result.

### Webhook Triggers

```sql
CREATE TABLE IF NOT EXISTS webhook_triggers (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    endpoint_path TEXT NOT NULL,
    secret_key TEXT,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (job_id) REFERENCES scheduler_jobs(id) ON DELETE CASCADE
);
```

Defines webhook endpoints that can trigger scheduled jobs.

## Authentication Tables

### Users

```sql
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    display_name TEXT,
    password_hash TEXT, -- Only for local authentication
    external_id TEXT, -- For Keycloak/external auth
    auth_provider TEXT NOT NULL DEFAULT 'local', -- 'local', 'keycloak', etc.
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

Stores user accounts, supporting both local authentication and external providers like Keycloak.

### Roles

```sql
CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

Defines roles that can be assigned to users.

### Permissions

```sql
CREATE TABLE IF NOT EXISTS permissions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    resource TEXT NOT NULL, -- The resource this permission applies to
    action TEXT NOT NULL, -- 'read', 'write', 'execute', etc.
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

Defines granular permissions that can be assigned to roles.

### User Roles

```sql
CREATE TABLE IF NOT EXISTS user_roles (
    user_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);
```

Associates users with roles in a many-to-many relationship.

### Role Permissions

```sql
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id TEXT NOT NULL,
    permission_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (role_id, permission_id),
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);
```

Associates roles with permissions in a many-to-many relationship.

### Auth Sessions

```sql
CREATE TABLE IF NOT EXISTS auth_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    token_hash TEXT NOT NULL,
    refresh_token_hash TEXT,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_accessed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

Tracks active authentication sessions.

## Relationships

- A **Connector** can have multiple **Routes** and **Scheduler Jobs**
- A **Route** connects a **Connector** to **Routing Rules**
- **Routing Rules** apply **Transformations** and route to **Destinations**
- **Scheduler Jobs** have **Job Executions** and can be triggered via **Webhook Triggers**
- **Users** are assigned **Roles** through **User Roles**
- **Roles** are granted **Permissions** through **Role Permissions**
- **Users** can have multiple **Auth Sessions**

## Schema Migrations

Database schema migrations are managed in the `migrations/` directory at the project root. See the [README](../../migrations/README.md) in that directory for more details on how migrations are structured and applied. 