# Scheduler Guide

The Scheduler module in Muxly allows you to automate data tasks by scheduling them to run at specific times or intervals.

## Overview

The Scheduler provides the following capabilities:

- Schedule jobs using cron expressions
- Configure different actions for jobs (queries, exports, etc.)
- Monitor job execution status
- Retry failed jobs automatically
- Store job execution history

## Configuration

Scheduler configuration is defined in the main `config.yml` file under the `scheduler` section:

```yaml
scheduler:
  enabled: true
  job_storage: "memory"  # memory, database
  concurrent_jobs: 5
  poll_interval_seconds: 30
  max_retries: 3
  jobs:
    - id: "daily-sync"
      name: "Daily Data Sync"
      schedule: "0 0 * * *"  # Cron expression (midnight daily)
      enabled: true
      connector_id: "bigquery-main"
      action:
        type: "query"
        params:
          query: "SELECT * FROM sales.transactions WHERE date = CURRENT_DATE() - 1"
```

### Key Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `enabled` | Enable/disable scheduler | `true` |
| `job_storage` | Where to store job information | `memory` |
| `concurrent_jobs` | Maximum number of jobs to run simultaneously | `5` |
| `poll_interval_seconds` | How often to check for jobs that need to run | `30` |
| `max_retries` | Maximum number of retries for failed jobs | `3` |

## Cron Expressions

Jobs are scheduled using cron expressions. A cron expression consists of 5 fields:

```
┌───────────── minute (0 - 59)
│ ┌───────────── hour (0 - 23)
│ │ ┌───────────── day of the month (1 - 31)
│ │ │ ┌───────────── month (1 - 12)
│ │ │ │ ┌───────────── day of the week (0 - 6) (Sunday to Saturday)
│ │ │ │ │
│ │ │ │ │
* * * * *
```

### Common Cron Examples

| Description | Cron Expression |
|-------------|----------------|
| Every minute | `* * * * *` |
| Every hour at minute 0 | `0 * * * *` |
| Every day at midnight | `0 0 * * *` |
| Every Monday at 9:00 AM | `0 9 * * 1` |
| Every 15 minutes | `*/15 * * * *` |
| At 2:30 PM on the first day of each month | `30 14 1 * *` |

### Special Expressions

Muxly also supports some special cron expressions:

| Expression | Description |
|------------|-------------|
| `@yearly` or `@annually` | Run once a year at midnight on January 1 (`0 0 1 1 *`) |
| `@monthly` | Run once a month at midnight on the first day (`0 0 1 * *`) |
| `@weekly` | Run once a week at midnight on Sunday (`0 0 * * 0`) |
| `@daily` or `@midnight` | Run once a day at midnight (`0 0 * * *`) |
| `@hourly` | Run once an hour at the beginning of the hour (`0 * * * *`) |

## Job Configuration

Each job in the scheduler requires the following configuration:

```yaml
jobs:
  - id: "daily-sync"  # Unique identifier
    name: "Daily Data Sync"  # Human-readable name
    schedule: "0 0 * * *"  # Cron expression
    enabled: true  # Enable/disable this job
    connector_id: "bigquery-main"  # ID of connector to use
    action:  # Action to perform
      type: "query"  # Type of action
      params:  # Action parameters
        query: "SELECT * FROM sales.transactions WHERE date = CURRENT_DATE() - 1"
```

### Job Actions

Muxly supports different types of job actions:

#### Query Action

Executes a query on the specified connector:

```yaml
action:
  type: "query"
  params:
    query: "SELECT * FROM sales.transactions WHERE date = CURRENT_DATE() - 1"
    parameters:  # Optional query parameters
      - name: "start_date"
        type: "DATE"
        value: "2023-01-01"
```

#### Export Action

Exports data to a specified destination:

```yaml
action:
  type: "export"
  params:
    data_spec:  # What data to export (varies by connector type)
      query: "SELECT * FROM sales.transactions WHERE date = CURRENT_DATE() - 1"
    destination:
      type: "file"  # file, s3, webhook, etc.
      config:
        path: "/exports/data_${DATE}.csv"
        format: "csv"  # csv, json, parquet
```

#### Pipeline Action

Executes a predefined data pipeline:

```yaml
action:
  type: "pipeline"
  params:
    pipeline_id: "sales-to-marketing"  # ID of a defined route in the router
    run_params:  # Optional parameters to pass to the pipeline
      date: "${DATE}"
```

### Dynamic Date Variables

The scheduler supports the following dynamic date variables that can be used in actions:

| Variable | Description | Example |
|----------|-------------|---------|
| `${DATE}` | Current date (YYYY-MM-DD) | `2023-04-15` |
| `${YESTERDAY}` | Yesterday's date (YYYY-MM-DD) | `2023-04-14` |
| `${TIMESTAMP}` | Current timestamp (YYYYMMDD_HHMMSS) | `20230415_120000` |
| `${YEAR}` | Current year (YYYY) | `2023` |
| `${MONTH}` | Current month (MM) | `04` |
| `${DAY}` | Current day (DD) | `15` |

Example usage:
```yaml
query: "SELECT * FROM sales.transactions WHERE date = '${YESTERDAY}'"
```

## Job Storage

The scheduler can store job execution information in different backends:

### Memory Storage

Stores job execution history in memory. Simple but doesn't persist across restarts:

```yaml
scheduler:
  job_storage: "memory"
```

### Database Storage

Stores job execution history in a database for persistence:

```yaml
scheduler:
  job_storage: "database"
  storage_config:
    connection_string: "${DATABASE_URL}"
    table_prefix: "scheduler_"  # Optional prefix for scheduler tables
```

## Monitoring Jobs

You can monitor scheduled jobs through the API or web interface:

### API Endpoints

- `GET /api/v1/scheduler/jobs` - List all configured jobs
- `GET /api/v1/scheduler/jobs/{job_id}` - Get details of a specific job
- `GET /api/v1/scheduler/jobs/{job_id}/runs` - Get execution history of a job
- `POST /api/v1/scheduler/jobs/{job_id}/run` - Manually trigger a job
- `PUT /api/v1/scheduler/jobs/{job_id}/enable` - Enable a job
- `PUT /api/v1/scheduler/jobs/{job_id}/disable` - Disable a job

### Web Interface

The Muxly web interface provides a dashboard for monitoring scheduled jobs, including:

- Job status and next scheduled run time
- Execution history with success/failure status
- Job output and error logs
- Manual run capability
- Enable/disable functionality

## Advanced Usage

### Job Dependencies

You can create job dependencies by using the `depends_on` field:

```yaml
jobs:
  - id: "extract-data"
    name: "Extract Data"
    schedule: "0 0 * * *"
    # ... other configuration ...
  
  - id: "transform-data"
    name: "Transform Data"
    depends_on: "extract-data"  # This job will run after extract-data
    # No schedule needed for dependent jobs
    # ... other configuration ...
```

### Notification on Job Completion

Configure notifications for job completion or failure:

```yaml
jobs:
  - id: "daily-sync"
    # ... other configuration ...
    notifications:
      on_success:
        - type: "email"
          recipients: ["admin@example.com"]
          subject: "Job Successful: ${job_name}"
          message: "Job ${job_id} completed successfully at ${completion_time}"
      on_failure:
        - type: "slack"
          webhook_url: "${SLACK_WEBHOOK_URL}"
          channel: "#alerts"
          message: "🚨 Job ${job_id} failed: ${error_message}"
```

### Run Conditions

You can add conditions that determine whether a job should run:

```yaml
jobs:
  - id: "conditional-job"
    # ... other configuration ...
    run_condition:
      type: "query"
      connector_id: "bigquery-main"
      query: "SELECT COUNT(*) > 0 FROM sales.transactions WHERE date = CURRENT_DATE() - 1"
```

## Best Practices

1. **Use descriptive job IDs and names** to easily identify jobs.
2. **Start with longer intervals** and refine as needed to avoid excessive resource usage.
3. **Set appropriate retry settings** based on the nature of the job.
4. **Use database storage** for production environments to ensure job history persistence.
5. **Monitor job execution time** to optimize resource usage.
6. **Use variables** instead of hardcoded dates/values to make jobs more dynamic.
7. **Implement notifications** for critical jobs to be alerted on failures.
8. **Avoid scheduling too many jobs** at the same time to prevent resource contention.

## Troubleshooting

### Common Issues

#### Jobs Not Running

- Verify the scheduler is enabled
- Check if the specific job is enabled
- Validate the cron expression
- Check if the connector exists and is properly configured
- Verify the server time zone matches your expectations

#### Job Failures

- Check connector authentication
- Validate query syntax
- Verify destination permissions
- Check network connectivity

#### Performance Issues

- Reduce the number of concurrent jobs
- Optimize queries to reduce execution time
- Consider using database storage for better management 