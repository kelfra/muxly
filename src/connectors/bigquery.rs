use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use gcp_auth::{AuthenticationManager, CustomServiceAccount};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use reqwest::{Client, header};

use crate::connectors::base::{
    AuthSettings, Connector, ConnectorData, ConnectorSettings, ConnectionStatus,
};

/// BigQuery-specific connection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigQueryConnectionSettings {
    /// Google Cloud project ID
    pub project_id: String,
    /// Default dataset ID
    pub dataset_id: Option<String>,
    /// Location/region for queries
    pub location: Option<String>,
    /// Maximum rows to return per request
    pub max_results: Option<u32>,
}

/// BigQuery Connector implementation
pub struct BigQueryConnector {
    /// Connector identifier
    id: String,
    /// Human-readable name
    name: String,
    /// Whether this connector is enabled
    enabled: bool,
    /// Authentication settings
    auth: AuthSettings,
    /// Connection settings specific to BigQuery
    connection: BigQueryConnectionSettings,
    /// HTTP client for API requests
    client: Client,
    /// Authentication token
    auth_token: Option<String>,
}

impl BigQueryConnector {
    /// Create a new BigQuery connector
    pub fn new() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            enabled: false,
            auth: AuthSettings {
                auth_type: String::new(),
                params: HashMap::new(),
            },
            connection: BigQueryConnectionSettings {
                project_id: String::new(),
                dataset_id: None,
                location: None,
                max_results: Some(1000),
            },
            client: Client::new(),
            auth_token: None,
        }
    }

    /// Authenticate with Google Cloud Platform
    async fn authenticate(&mut self) -> Result<String> {
        match self.auth.auth_type.as_str() {
            "service_account" => {
                // Get the service account JSON from the params
                let service_account_json = self
                    .auth
                    .params
                    .get("service_account_json")
                    .ok_or_else(|| anyhow!("Missing service_account_json parameter"))?;

                let service_account: CustomServiceAccount = serde_json::from_value(service_account_json.clone())?;
                let auth_manager = AuthenticationManager::from(service_account);
                let token = auth_manager
                    .get_token(&["https://www.googleapis.com/auth/bigquery"])
                    .await?;

                Ok(token.as_str().to_string())
            }
            "application_default" => {
                // Use application default credentials
                let auth_manager = AuthenticationManager::new().await?;
                let token = auth_manager
                    .get_token(&["https://www.googleapis.com/auth/bigquery"])
                    .await?;

                Ok(token.as_str().to_string())
            }
            _ => Err(anyhow!(
                "Unsupported authentication type: {}",
                self.auth.auth_type
            )),
        }
    }

    /// Execute a BigQuery SQL query
    async fn execute_query(&self, query: &str, params: Option<Value>) -> Result<Value> {
        let token = self.auth_token.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;
        
        // Create the request payload
        let mut request_json = json!({
            "query": query,
            "useLegacySql": false,
        });
        
        // Add query parameters if provided
        if let Some(params_value) = params {
            request_json["queryParameters"] = params_value;
        }
        
        // Add location if specified
        if let Some(location) = &self.connection.location {
            request_json["location"] = json!(location);
        }
        
        // Add maximum results if specified
        if let Some(max_results) = self.connection.max_results {
            request_json["maxResults"] = json!(max_results);
        }
        
        // Build the request URL
        let url = format!(
            "https://bigquery.googleapis.com/bigquery/v2/projects/{}/queries",
            self.connection.project_id
        );
        
        // Execute the request
        let response = self.client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request_json)
            .send()
            .await?;
        
        // Check for errors
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("BigQuery API error: {}", error_text));
        }
        
        // Parse the response
        let response_data: Value = response.json().await?;
        
        Ok(response_data)
    }

    /// Transform BigQuery API response into a more usable format
    fn transform_query_result(&self, raw_result: Value) -> Result<Value> {
        // Extract schema information
        let schema = raw_result["schema"]["fields"]
            .as_array()
            .ok_or_else(|| anyhow!("Missing schema fields"))?;
        
        // Extract row data
        let rows = match raw_result["rows"].as_array() {
            Some(rows) => rows,
            None => return Ok(json!([])), // No rows returned
        };
        
        // Transform rows into a more usable format
        let mut transformed_rows = Vec::new();
        
        for row in rows {
            let values = row["f"]
                .as_array()
                .ok_or_else(|| anyhow!("Invalid row format"))?;
            
            let mut row_obj = serde_json::Map::new();
            
            for (i, field) in schema.iter().enumerate() {
                if i >= values.len() {
                    continue;
                }
                
                let field_name = field["name"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Invalid field name"))?;
                
                let value = &values[i]["v"];
                row_obj.insert(field_name.to_string(), value.clone());
            }
            
            transformed_rows.push(Value::Object(row_obj));
        }
        
        Ok(json!(transformed_rows))
    }
}

#[async_trait]
impl Connector for BigQueryConnector {
    fn get_type(&self) -> &str {
        "bigquery"
    }
    
    fn get_id(&self) -> &str {
        &self.id
    }
    
    async fn initialize(&mut self, settings: ConnectorSettings) -> Result<()> {
        // Extract settings
        self.id = settings.id;
        self.name = settings.name;
        self.enabled = settings.enabled;
        self.auth = settings.auth;
        
        // Parse connection settings
        let connection_value = serde_json::to_value(&settings.connection)?;
        self.connection = serde_json::from_value(connection_value)?;
        
        // Validate required settings
        if self.connection.project_id.is_empty() {
            return Err(anyhow!("Missing required parameter: project_id"));
        }
        
        // Perform initial authentication
        let token = self.authenticate().await?;
        self.auth_token = Some(token);
        
        Ok(())
    }
    
    async fn test_connection(&self) -> Result<ConnectionStatus> {
        if !self.enabled {
            return Ok(ConnectionStatus::Disconnected);
        }
        
        // Simple test query to check connection
        let test_query = "SELECT 1 as test_value";
        
        match self.execute_query(test_query, None).await {
            Ok(_) => Ok(ConnectionStatus::Connected),
            Err(e) => {
                if e.to_string().contains("Invalid Credentials") {
                    Ok(ConnectionStatus::CredentialsInvalid)
                } else if e.to_string().contains("Rate Limit") {
                    Ok(ConnectionStatus::RateLimited)
                } else {
                    Ok(ConnectionStatus::Error(e.to_string()))
                }
            }
        }
    }
    
    async fn fetch_data(&self, params: Value) -> Result<ConnectorData> {
        // Extract query from params
        let query = params["query"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing required parameter: query"))?;
        
        // Get query parameters if provided
        let query_params = params.get("parameters").cloned();
        
        // Execute the query
        let raw_result = self.execute_query(query, query_params).await?;
        
        // Transform the result
        let transformed_data = self.transform_query_result(raw_result.clone())?;
        
        // Create metadata
        let mut metadata = HashMap::new();
        
        // Add information about total rows, bytes processed, etc.
        if let Some(total_rows) = raw_result["totalRows"].as_str() {
            metadata.insert("total_rows".to_string(), json!(total_rows));
        }
        
        if let Some(bytes_processed) = raw_result["totalBytesProcessed"].as_str() {
            metadata.insert("bytes_processed".to_string(), json!(bytes_processed));
        }
        
        if let Some(cache_hit) = raw_result["cacheHit"].as_bool() {
            metadata.insert("cache_hit".to_string(), json!(cache_hit));
        }
        
        // Return the connector data
        Ok(ConnectorData {
            connector_id: self.id.clone(),
            timestamp: Utc::now(),
            data: transformed_data,
            metadata,
        })
    }
    
    async fn get_metadata(&self) -> Result<Value> {
        // Return information about available datasets and tables
        if !self.enabled {
            return Err(anyhow!("Connector is not enabled"));
        }
        
        let token = self.auth_token.as_ref().ok_or_else(|| anyhow!("Not authenticated"))?;
        
        // Get datasets in the project
        let datasets_url = format!(
            "https://bigquery.googleapis.com/bigquery/v2/projects/{}/datasets",
            self.connection.project_id
        );
        
        let datasets_response = self.client
            .get(&datasets_url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;
        
        if !datasets_response.status().is_success() {
            let error_text = datasets_response.text().await?;
            return Err(anyhow!("BigQuery API error: {}", error_text));
        }
        
        let datasets_data: Value = datasets_response.json().await?;
        let datasets = datasets_data["datasets"].as_array().unwrap_or(&vec![]);
        
        let mut metadata = json!({
            "project_id": self.connection.project_id,
            "datasets": []
        });
        
        let datasets_array = metadata["datasets"].as_array_mut().unwrap();
        
        for dataset in datasets {
            let dataset_id = dataset["datasetReference"]["datasetId"]
                .as_str()
                .unwrap_or("unknown");
            
            datasets_array.push(json!({
                "dataset_id": dataset_id,
                "tables": []
            }));
        }
        
        Ok(metadata)
    }
    
    fn get_configuration_template(&self) -> Value {
        json!({
            "connection": {
                "project_id": {
                    "type": "string",
                    "required": true,
                    "description": "Google Cloud Project ID"
                },
                "dataset_id": {
                    "type": "string",
                    "required": false,
                    "description": "Default BigQuery dataset ID"
                },
                "location": {
                    "type": "string",
                    "required": false,
                    "description": "BigQuery location/region (e.g., US, EU)"
                },
                "max_results": {
                    "type": "integer",
                    "required": false,
                    "default": 1000,
                    "description": "Maximum number of results to return per query"
                }
            },
            "auth": {
                "auth_type": {
                    "type": "string",
                    "required": true,
                    "enum": ["service_account", "application_default"],
                    "description": "Authentication method to use"
                },
                "service_account_json": {
                    "type": "object",
                    "required": true,
                    "condition": "auth_type === 'service_account'",
                    "description": "Service account JSON key file contents"
                }
            }
        })
    }
}
