-- Revert Scheduler jobs migration

DROP INDEX IF EXISTS idx_job_executions_status;
DROP TABLE IF EXISTS webhook_triggers;
DROP TABLE IF EXISTS job_executions;
DROP TABLE IF EXISTS scheduler_jobs; 