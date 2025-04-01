# Database Migrations

This directory contains all database schema migrations for the Muxly project. Migrations are applied in order by timestamp, using the SQLx migrations framework.

## Migration Structure

- Each migration is in a separate directory named with the format `YYYYMMDDHHMMSS_name`
- Each migration directory contains two SQL files:
  - `up.sql` - Applied when migrating forward
  - `down.sql` - Applied when rolling back a migration

## Migration Order

1. **20230501000000_initial** - Initial schema with connectors, schedules, and basic tables
2. **20230502000000_router_config** - Router configuration tables for destinations and rules
3. **20230503000000_scheduler_jobs** - Scheduler jobs and executions tables
4. **20230504000000_auth_tables** - Authentication tables for users, roles, and permissions

## Running Migrations

Migrations are automatically applied on application startup. The code for this is in `src/storage/migrations/mod.rs`.

## Adding New Migrations

To add a new migration:

1. Create a new directory with a timestamp prefix and descriptive name:
   ```
   mkdir migrations/$(date +%Y%m%d%H%M%S)_description
   ```

2. Create the `up.sql` and `down.sql` files in this directory.

3. The `up.sql` file should contain the SQL to apply the migration.

4. The `down.sql` file should contain the SQL to reverse the migration.

## Schema Documentation

For detailed documentation of the database schema, refer to the [Database Schema Guide](../docs/development/database_schema.md). 