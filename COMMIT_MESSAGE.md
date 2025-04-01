feat(connectors): Implement Connectors Module with BigQuery, GA4, and HubSpot integrations

This commit implements the Connectors Module, which enables integration with 
third-party data sources. It includes:

- Core connector infrastructure with a common interface
- BigQuery connector for SQL-based data extraction
- GA4 connector for Google Analytics metrics
- HubSpot connector for CRM data
- Plugin system for custom connector extensions

Closes #XXX 