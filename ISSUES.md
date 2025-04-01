# Known Issues - Connectors Module

## Compilation Errors

1. **Project-Wide Compilation Errors**
   - There are compilation errors in the project, but most are related to other modules that reference API routes and handlers not yet implemented
   - These errors are expected and will be resolved as we implement subsequent modules

2. **Cargo.toml Dependencies**
   - Dependency versions may need adjustment for better compatibility
   - Some dependencies might need feature flags to be enabled/disabled

## Connector-Specific Issues

3. **BigQuery Connector**
   - The authentication implementation for service accounts needs proper testing with actual credentials
   - Query parameter binding is implemented but needs validation with complex query scenarios
   - Error handling for specific BigQuery error codes could be improved

4. **GA4 Connector**
   - The service account authentication method is a simplified placeholder that needs to be fully implemented
   - Rate limiting detection and handling could be enhanced with exponential backoff
   - Support for advanced GA4 report types (funnel visualization, etc.) is not yet implemented

5. **HubSpot Connector**
   - The filter query construction for object filtering could be improved for complex filter scenarios
   - Batch operations for creating/updating multiple records are not yet implemented
   - Webhook registration for real-time data updates is planned but not implemented

6. **Plugin System**
   - The dynamic loading mechanism works but lacks robust error reporting for plugin compatibility issues
   - No sandboxing or security measures for untrusted plugins
   - Plugin versioning and compatibility checking is minimal

## Future Improvements

7. **Authentication Improvements**
   - Token caching and persistence between application restarts
   - More secure credential storage with encryption at rest
   - Support for additional OAuth flows (device flow, PKCE)

8. **Performance Optimizations**
   - Implement connection pooling for connector instances
   - Add caching layer for frequently accessed data
   - Optimize large data transfer with streaming and pagination

9. **Error Handling and Reporting**
   - More detailed error reporting with actionable suggestions
   - Improved logging of connection issues and retries
   - Structured error responses for API consumers

10. **Documentation**
    - API documentation with examples needs improvement
    - Configuration templates should include more examples
    - Plugin development guide should be expanded

## Next Steps

These issues will be addressed in subsequent development cycles, with priorities determined by their impact on core functionality. Some of these issues may be addressed as part of the Router Module implementation in Phase 3. 