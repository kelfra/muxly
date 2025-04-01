-- Router configuration migration

-- Create destinations table
CREATE TABLE IF NOT EXISTS destinations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    destination_type TEXT NOT NULL, -- 'database', 'email', 'file', 'prometheus', 's3', 'slack', 'webhook'
    config TEXT NOT NULL, -- JSON blob for destination configuration
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create routing_rules table
CREATE TABLE IF NOT EXISTS routing_rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 100,
    enabled BOOLEAN NOT NULL DEFAULT true,
    condition TEXT, -- JSON blob for rule condition
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create rule_transformations table
CREATE TABLE IF NOT EXISTS rule_transformations (
    id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL,
    transformation_type TEXT NOT NULL, -- 'rename_field', 'filter', 'set_field', etc.
    config TEXT NOT NULL, -- JSON blob for transformation configuration
    sequence_order INTEGER NOT NULL DEFAULT 0, -- Order of application
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (rule_id) REFERENCES routing_rules(id) ON DELETE CASCADE
);

-- Create rule_destinations table for many-to-many relationship
CREATE TABLE IF NOT EXISTS rule_destinations (
    rule_id TEXT NOT NULL,
    destination_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (rule_id, destination_id),
    FOREIGN KEY (rule_id) REFERENCES routing_rules(id) ON DELETE CASCADE,
    FOREIGN KEY (destination_id) REFERENCES destinations(id) ON DELETE CASCADE
);

-- Create routes table
CREATE TABLE IF NOT EXISTS routes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    connector_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    error_handling TEXT NOT NULL DEFAULT '{"mode": "continue"}', -- JSON blob for error handling configuration
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (connector_id) REFERENCES connectors(id) ON DELETE CASCADE
);

-- Create route_rules table for many-to-many relationship
CREATE TABLE IF NOT EXISTS route_rules (
    route_id TEXT NOT NULL,
    rule_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (route_id, rule_id),
    FOREIGN KEY (route_id) REFERENCES routes(id) ON DELETE CASCADE,
    FOREIGN KEY (rule_id) REFERENCES routing_rules(id) ON DELETE CASCADE
); 