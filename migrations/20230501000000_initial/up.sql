-- Initial database migration for Muxly

-- Create connectors table
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

-- Create schedules table
CREATE TABLE IF NOT EXISTS schedules (
    id TEXT PRIMARY KEY,
    connector_id TEXT NOT NULL,
    schedule_type TEXT NOT NULL, -- 'cron', 'webhook', 'api'
    settings TEXT NOT NULL, -- JSON blob for schedule settings
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (connector_id) REFERENCES connectors(id) ON DELETE CASCADE
);

-- Create transformations table
CREATE TABLE IF NOT EXISTS transformations (
    id TEXT PRIMARY KEY,
    connector_id TEXT NOT NULL,
    settings TEXT NOT NULL, -- JSON blob for transformation settings
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (connector_id) REFERENCES connectors(id) ON DELETE CASCADE
);

-- Create outputs table
CREATE TABLE IF NOT EXISTS outputs (
    id TEXT PRIMARY KEY,
    output_type TEXT NOT NULL, -- 'file', 'prometheus', 'webhook', etc.
    settings TEXT NOT NULL, -- JSON blob for output settings
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create connector_outputs table for many-to-many relationship
CREATE TABLE IF NOT EXISTS connector_outputs (
    connector_id TEXT NOT NULL,
    output_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (connector_id, output_id),
    FOREIGN KEY (connector_id) REFERENCES connectors(id) ON DELETE CASCADE,
    FOREIGN KEY (output_id) REFERENCES outputs(id) ON DELETE CASCADE
);

-- Create sync_history table
CREATE TABLE IF NOT EXISTS sync_history (
    id TEXT PRIMARY KEY,
    connector_id TEXT NOT NULL,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    status TEXT NOT NULL, -- 'running', 'success', 'error'
    records_processed INTEGER,
    error_message TEXT,
    details TEXT, -- JSON blob for additional details
    FOREIGN KEY (connector_id) REFERENCES connectors(id) ON DELETE CASCADE
);

-- Create settings table for global settings
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert default settings
INSERT INTO settings (key, value) VALUES ('version', '0.1.0'); 