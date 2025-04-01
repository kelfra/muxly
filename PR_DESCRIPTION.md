# Feature: Connectors Module Implementation

## Overview
This PR implements the Connectors Module, a key component of the Muxly system that enables integration with various third-party data sources. This implementation completes Phase 2 of our project roadmap.

## Changes
- Added comprehensive connector infrastructure with common traits and interfaces
- Implemented three connector types:
  - BigQuery connector for Google BigQuery integration
  - GA4 connector for Google Analytics 4 metrics
  - HubSpot connector for CRM data
- Created a plugin system for custom connector extensions
- Consolidated and expanded project documentation
- Created comprehensive user and developer guides

## Technical Details

### Base Connector Infrastructure
- Defined a `Connector` trait with standard interface for all data sources
- Implemented authentication handling for various auth methods
- Created common error handling and rate limiting mechanisms
- Added connection testing and validation capabilities

### BigQuery Connector
- Implemented Google Cloud authentication (OAuth and service accounts)
- Added SQL query execution with parameter support
- Implemented data transformation for query results
- Added metadata retrieval for datasets and tables

### GA4 Connector
- Implemented OAuth authentication flow with token refresh
- Added report generation with dimensions and metrics
- Created data transformation for GA4's complex response format
- Implemented rate limiting and retry mechanisms

### HubSpot Connector
- Implemented OAuth and API key authentication
- Added support for contacts, companies, deals, and other CRM objects
- Implemented association management between objects
- Created data transformation for HubSpot's API responses

### Plugin System
- Implemented dynamic loading of custom connectors using `libloading`
- Created a stable plugin API for third-party developers
- Added type-safe interfaces for plugin registration
- Implemented example plugin to demonstrate the API

## Documentation Improvements
- Completely restructured documentation for better organization:
  - Created separate directories for user and developer documentation
  - Consolidated duplicate documentation files
  - Created a unified README for documentation navigation
- Created comprehensive user guides:
  - Configuration Guide with examples for all settings
  - Connectors Guide for data source configuration
  - Scheduler Guide for job scheduling
  - Router Guide for data destination configuration
- Enhanced developer documentation:
  - Detailed implementation notes for connectors
  - Known issues and future improvements
  - Updated implementation plan

## Testing
- Thorough unit tests for all connector implementations
- Authentication and error handling test cases
- Data transformation validation tests
- Plugin system validation tests

## Dependencies
- Added `gcp_auth` for Google Cloud authentication
- Added `libloading` for the plugin system
- Added `oauth2` for OAuth authentication flows
- Added other supporting libraries for HTTP requests and JSON processing

## Next Steps
The next phase (Phase 3) will focus on implementing the Router Module to handle data destination routing. 