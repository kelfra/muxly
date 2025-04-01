-- Down migration for the initial database setup

-- Drop tables in reverse order of creation (to handle foreign key constraints)
DROP TABLE IF EXISTS settings;
DROP TABLE IF EXISTS sync_history;
DROP TABLE IF EXISTS connector_outputs;
DROP TABLE IF EXISTS outputs;
DROP TABLE IF EXISTS transformations;
DROP TABLE IF EXISTS schedules;
DROP TABLE IF EXISTS connectors; 