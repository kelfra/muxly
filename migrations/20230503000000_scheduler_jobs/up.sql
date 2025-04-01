-- Scheduler jobs migration

-- Create scheduler_jobs table
CREATE TABLE IF NOT EXISTS scheduler_jobs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    job_type TEXT NOT NULL, -- 'cron', 'webhook', 'api'
    schedule TEXT, -- cron expression for cron jobs
    connector_id TEXT NOT NULL,
    action_type TEXT NOT NULL, -- 'query', 'export', etc.
    action_params TEXT NOT NULL, -- JSON blob with action parameters
    enabled BOOLEAN NOT NULL DEFAULT true,
    max_retries INTEGER NOT NULL DEFAULT 3,
    backoff_strategy TEXT NOT NULL DEFAULT 'exponential',
    timeout_seconds INTEGER NOT NULL DEFAULT 300,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (connector_id) REFERENCES connectors(id) ON DELETE CASCADE
);

-- Create job_executions table
CREATE TABLE IF NOT EXISTS job_executions (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    status TEXT NOT NULL, -- 'running', 'success', 'error', 'timeout', 'cancelled'
    result TEXT, -- JSON blob with execution result or error
    attempt INTEGER NOT NULL DEFAULT 1,
    triggered_by TEXT NOT NULL DEFAULT 'schedule', -- 'schedule', 'manual', 'webhook'
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (job_id) REFERENCES scheduler_jobs(id) ON DELETE CASCADE
);

-- Create webhook_triggers table
CREATE TABLE IF NOT EXISTS webhook_triggers (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    endpoint_path TEXT NOT NULL,
    secret_key TEXT, -- For HMAC validation
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (job_id) REFERENCES scheduler_jobs(id) ON DELETE CASCADE
);

-- Index for faster retrieval of pending jobs
CREATE INDEX IF NOT EXISTS idx_job_executions_status ON job_executions(status); 