# Known Issues

## Project-Wide Issues

1. **Compilation Errors**
   - There are compilation errors in the project, but most are related to other modules that reference API routes and handlers not yet implemented
   - These errors are expected and will be resolved as we implement subsequent modules

2. **Dependencies**
   - Dependency versions may need adjustment for better compatibility
   - Some dependencies might need feature flags to be enabled/disabled
   - Cargo.toml should be regularly updated to ensure compatibility with the latest Rust version

3. **API Implementation**
   - Some middleware implementations need updating to match the latest Axum version
   - The main application has references to components that will be implemented in future phases

## Module-Specific Issues

### Connectors Module

1. **BigQuery Connector**
   - The authentication implementation for service accounts needs proper testing with actual credentials
   - Query parameter binding is implemented but needs validation with complex query scenarios
   - Error handling for specific BigQuery error codes could be improved

2. **GA4 Connector**
   - The service account authentication method is a simplified placeholder that needs to be fully implemented
   - Rate limiting detection and handling could be enhanced with exponential backoff
   - Support for advanced GA4 report types (funnel visualization, etc.) is not yet implemented

3. **HubSpot Connector**
   - The filter query construction for object filtering could be improved for complex filter scenarios
   - Batch operations for creating/updating multiple records are not yet implemented
   - Webhook registration for real-time data updates is planned but not implemented

4. **Plugin System**
   - The dynamic loading mechanism works but lacks robust error reporting for plugin compatibility issues
   - No sandboxing or security measures for untrusted plugins
   - Plugin versioning and compatibility checking is minimal

### Scheduler Module

5. **Job Persistence**
   - Jobs are currently stored in memory and not persisted to a database
   - Job status and history is lost on application restart
   - No distributed coordination for multi-instance deployments

### Configuration Module

6. **Validation**
   - More comprehensive validation for complex configuration structures needed
   - Better error messages for misconfiguration

## Future Improvements

### Authentication Improvements
- Token caching and persistence between application restarts
- More secure credential storage with encryption at rest
- Support for additional OAuth flows (device flow, PKCE)

### Performance Optimizations
- Implement connection pooling for connector instances
- Add caching layer for frequently accessed data
- Optimize large data transfer with streaming and pagination

### Error Handling and Reporting
- More detailed error reporting with actionable suggestions
- Improved logging of connection issues and retries
- Structured error responses for API consumers

### Documentation
- API documentation with examples needs improvement
- Configuration templates should include more examples
- Plugin development guide should be expanded

## Next Steps

These issues will be addressed in subsequent development cycles, with priorities determined by their impact on core functionality. Many of these issues will be addressed as part of the Router Module implementation in Phase 3. 