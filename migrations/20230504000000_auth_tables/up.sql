-- Authentication tables migration

-- Create users table
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

-- Create roles table
CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create user_roles table for many-to-many relationship
CREATE TABLE IF NOT EXISTS user_roles (
    user_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);

-- Create permissions table
CREATE TABLE IF NOT EXISTS permissions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    resource TEXT NOT NULL, -- The resource this permission applies to
    action TEXT NOT NULL, -- 'read', 'write', 'execute', etc.
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create role_permissions table for many-to-many relationship
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id TEXT NOT NULL,
    permission_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (role_id, permission_id),
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);

-- Create auth_sessions table
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

-- Create default roles
INSERT INTO roles (id, name, description) 
VALUES 
    ('role_admin', 'admin', 'Administrator with full access'),
    ('role_user', 'user', 'Regular user with limited access'),
    ('role_readonly', 'readonly', 'Read-only access to the system');

-- Create permissions
INSERT INTO permissions (id, name, description, resource, action)
VALUES
    ('perm_admin_all', 'admin:all', 'Full administrative access', '*', '*'),
    ('perm_read_all', 'read:all', 'Read access to all resources', '*', 'read'),
    ('perm_manage_connectors', 'manage:connectors', 'Manage connectors', 'connectors', '*'),
    ('perm_manage_routes', 'manage:routes', 'Manage routes', 'routes', '*'),
    ('perm_manage_jobs', 'manage:jobs', 'Manage jobs', 'jobs', '*'),
    ('perm_manage_users', 'manage:users', 'Manage users', 'users', '*');

-- Associate permissions with roles
INSERT INTO role_permissions (role_id, permission_id)
VALUES
    ('role_admin', 'perm_admin_all'),
    ('role_user', 'perm_read_all'),
    ('role_user', 'perm_manage_connectors'),
    ('role_user', 'perm_manage_routes'),
    ('role_user', 'perm_manage_jobs'),
    ('role_readonly', 'perm_read_all');

-- Create indexes for faster queries
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_external_id ON users(external_id);
CREATE INDEX IF NOT EXISTS idx_auth_sessions_user_id ON auth_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_auth_sessions_expires_at ON auth_sessions(expires_at); 