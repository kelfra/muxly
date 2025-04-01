use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use reqwest::{Client, header};
use tokio::time::{sleep, Duration};

use crate::connectors::base::{
    AuthSettings, Connector, ConnectorData, ConnectorSettings, ConnectionStatus,
};

/// GA4-specific connection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GA4ConnectionSettings {
    /// GA4 property ID
    pub property_id: String,
    /// Default date range in days (from today)
    pub default_date_range_days: Option<u32>,
    /// Sampling level for reports
    pub sampling_level: Option<String>,
    /// Currency for reports
    pub currency: Option<String>,
}

/// GA4 Connector implementation
pub struct GA4Connector {
    /// Connector identifier
    id: String,
    /// Human-readable name
    name: String,
    /// Whether this connector is enabled
    enabled: bool,
    /// Authentication settings
    auth: AuthSettings,
    /// Connection settings specific to GA4
    connection: GA4ConnectionSettings,
    /// HTTP client for API requests
    client: Client,
    /// Authentication token
    access_token: Option<String>,
    /// Token expiration time
    token_expiry: Option<chrono::DateTime<chrono::Utc>>,
    /// Refresh token for OAuth
    refresh_token: Option<String>,
}

impl GA4Connector {
    /// Create a new GA4 connector
    pub fn new() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            enabled: false,
            auth: AuthSettings {
                auth_type: String::new(),
                params: HashMap::new(),
            },
            connection: GA4ConnectionSettings {
                property_id: String::new(),
                default_date_range_days: Some(30),
                sampling_level: Some("DEFAULT".to_string()),
                currency: Some("USD".to_string()),
            },
            client: Client::new(),
            access_token: None,
            token_expiry: None,
            refresh_token: None,
        }
    }

    /// Authenticate with Google Analytics API
    async fn authenticate(&mut self) -> Result<()> {
        match self.auth.auth_type.as_str() {
            "oauth" => {
                // Check if we already have a valid token
                if let (Some(expiry), Some(_)) = (self.token_expiry, &self.access_token) {
                    // Add a 5-minute buffer to token expiry
                    let buffer = chrono::Duration::minutes(5);
                    if expiry > Utc::now() + buffer {
                        return Ok(());
                    }
                }

                // If we have a refresh token, use it to get a new access token
                if let Some(refresh_token) = &self.refresh_token {
                    return self.refresh_access_token(refresh_token).await;
                }

                // Otherwise we need client ID and secret for initial authentication
                let client_id = self.auth.params.get("client_id")
                    .ok_or_else(|| anyhow!("Missing OAuth client_id"))?
                    .as_str()
                    .ok_or_else(|| anyhow!("client_id must be a string"))?;

                let client_secret = self.auth.params.get("client_secret")
                    .ok_or_else(|| anyhow!("Missing OAuth client_secret"))?
                    .as_str()
                    .ok_or_else(|| anyhow!("client_secret must be a string"))?;

                // For initial OAuth we need authorization code
                let auth_code = self.auth.params.get("auth_code")
                    .ok_or_else(|| anyhow!("Missing OAuth auth_code"))?
                    .as_str()
                    .ok_or_else(|| anyhow!("auth_code must be a string"))?;

                // Get initial tokens with authorization code
                let token_url = "https://oauth2.googleapis.com/token";
                let params = [
                    ("client_id", client_id),
                    ("client_secret", client_secret),
                    ("code", auth_code),
                    ("grant_type", "authorization_code"),
                    ("redirect_uri", "urn:ietf:wg:oauth:2.0:oob"),
                ];

                let response = self.client
                    .post(token_url)
                    .form(&params)
                    .send()
                    .await?;

                if !response.status().is_success() {
                    let error_text = response.text().await?;
                    return Err(anyhow!("OAuth error: {}", error_text));
                }

                let token_data: Value = response.json().await?;
                self.process_token_response(token_data)?;
                Ok(())
            }
            "service_account" => {
                // Get the service account JSON from the params
                let service_account_json = self.auth.params.get("service_account_json")
                    .ok_or_else(|| anyhow!("Missing service_account_json parameter"))?;

                // Use the google-auth-library-rs or gcp_auth crate to authenticate
                // This is a simplified example, in reality need to implement JWT auth flow
                let scopes = vec!["https://www.googleapis.com/auth/analytics.readonly"];
                
                // In a real implementation, we would create a JWT and sign it with the private key
                // from the service account JSON, then exchange it for an access token
                // For brevity, this implementation is simplified
                
                // Fake successful authentication (in reality, we'd make actual API call)
                self.access_token = Some("fake_token_for_example".to_string());
                self.token_expiry = Some(Utc::now() + chrono::Duration::hours(1));
                
                Ok(())
            }
            _ => Err(anyhow!("Unsupported authentication type: {}", self.auth.auth_type)),
        }
    }

    /// Refresh the access token using a refresh token
    async fn refresh_access_token(&mut self, refresh_token: &str) -> Result<()> {
        let client_id = self.auth.params.get("client_id")
            .ok_or_else(|| anyhow!("Missing OAuth client_id"))?
            .as_str()
            .ok_or_else(|| anyhow!("client_id must be a string"))?;

        let client_secret = self.auth.params.get("client_secret")
            .ok_or_else(|| anyhow!("Missing OAuth client_secret"))?
            .as_str()
            .ok_or_else(|| anyhow!("client_secret must be a string"))?;

        let token_url = "https://oauth2.googleapis.com/token";
        let params = [
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let response = self.client
            .post(token_url)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OAuth refresh error: {}", error_text));
        }

        let token_data: Value = response.json().await?;
        self.process_token_response(token_data)?;
        Ok(())
    }

    /// Process token response and extract relevant fields
    fn process_token_response(&mut self, token_data: Value) -> Result<()> {
        // Extract access token
        let access_token = token_data["access_token"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing access_token in response"))?
            .to_string();

        // Extract token expiration
        let expires_in = token_data["expires_in"]
            .as_u64()
            .ok_or_else(|| anyhow!("Missing expires_in in response"))?;

        // Calculate expiry time
        let expiry = Utc::now() + chrono::Duration::seconds(expires_in as i64);

        // Extract refresh token if present
        if let Some(refresh_token) = token_data["refresh_token"].as_str() {
            self.refresh_token = Some(refresh_token.to_string());
        }

        self.access_token = Some(access_token);
        self.token_expiry = Some(expiry);

        Ok(())
    }

    /// Create a data report request
    async fn run_report(&self, report_request: Value) -> Result<Value> {
        // Ensure we have an access token
        let access_token = self.access_token.as_ref()
            .ok_or_else(|| anyhow!("Not authenticated"))?;

        // Build the API URL
        let url = format!(
            "https://analyticsdata.googleapis.com/v1beta/properties/{}:runReport",
            self.connection.property_id
        );

        // Make the API request
        let response = self.client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&report_request)
            .send()
            .await?;

        // Check for rate limiting
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            // Extract retry-after header if present
            let retry_after = response.headers()
                .get(header::RETRY_AFTER)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(60); // Default to 60 seconds

            // Wait and retry
            sleep(Duration::from_secs(retry_after)).await;
            return self.run_report(report_request).await;
        }

        // Check for other errors
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("GA4 API error: {}", error_text));
        }

        // Parse the response
        let report_data: Value = response.json().await?;
        Ok(report_data)
    }

    /// Transform GA4 API response into a more usable format
    fn transform_report_data(&self, report_data: Value) -> Result<Value> {
        // Extract dimension headers
        let dimension_headers = match report_data["dimensionHeaders"].as_array() {
            Some(headers) => headers.iter()
                .map(|h| h["name"].as_str().unwrap_or("unknown").to_string())
                .collect::<Vec<String>>(),
            None => vec![],
        };

        // Extract metric headers
        let metric_headers = match report_data["metricHeaders"].as_array() {
            Some(headers) => headers.iter()
                .map(|h| h["name"].as_str().unwrap_or("unknown").to_string())
                .collect::<Vec<String>>(),
            None => vec![],
        };

        // Extract rows
        let rows = match report_data["rows"].as_array() {
            Some(row_data) => row_data,
            None => return Ok(json!([])), // No data
        };

        // Transform rows into a more usable format
        let mut transformed_rows = Vec::new();

        for row in rows {
            let mut row_obj = serde_json::Map::new();

            // Add dimensions
            if let Some(dimension_values) = row["dimensionValues"].as_array() {
                for (i, value) in dimension_values.iter().enumerate() {
                    if i < dimension_headers.len() {
                        let dimension_name = &dimension_headers[i];
                        let dimension_value = value["value"].as_str().unwrap_or("").to_string();
                        row_obj.insert(dimension_name.clone(), json!(dimension_value));
                    }
                }
            }

            // Add metrics
            if let Some(metric_values) = row["metricValues"].as_array() {
                for (i, value) in metric_values.iter().enumerate() {
                    if i < metric_headers.len() {
                        let metric_name = &metric_headers[i];
                        // Convert numeric values if possible
                        let metric_value = value["value"].as_str().unwrap_or("");
                        if let Ok(num) = metric_value.parse::<f64>() {
                            row_obj.insert(metric_name.clone(), json!(num));
                        } else {
                            row_obj.insert(metric_name.clone(), json!(metric_value));
                        }
                    }
                }
            }

            transformed_rows.push(Value::Object(row_obj));
        }

        Ok(json!(transformed_rows))
    }
}

#[async_trait]
impl Connector for GA4Connector {
    fn get_type(&self) -> &str {
        "ga4"
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
        if self.connection.property_id.is_empty() {
            return Err(anyhow!("Missing required parameter: property_id"));
        }
        
        // Extract refresh token if it exists in the auth params
        if let Some(refresh_token) = self.auth.params.get("refresh_token") {
            if let Some(token_str) = refresh_token.as_str() {
                self.refresh_token = Some(token_str.to_string());
            }
        }
        
        // Perform initial authentication
        self.authenticate().await?;
        
        Ok(())
    }
    
    async fn test_connection(&self) -> Result<ConnectionStatus> {
        if !self.enabled {
            return Ok(ConnectionStatus::Disconnected);
        }
        
        // Simple test request to check connection - request just one metric for minimal data
        let test_request = json!({
            "dateRanges": [{
                "startDate": "7daysAgo",
                "endDate": "today"
            }],
            "dimensions": [{"name": "date"}],
            "metrics": [{"name": "activeUsers"}],
            "limit": 1
        });
        
        match self.run_report(test_request).await {
            Ok(_) => Ok(ConnectionStatus::Connected),
            Err(e) => {
                if e.to_string().contains("Invalid Credentials") || e.to_string().contains("UNAUTHENTICATED") {
                    Ok(ConnectionStatus::CredentialsInvalid)
                } else if e.to_string().contains("TOO_MANY_REQUESTS") || e.to_string().contains("Rate Limit") {
                    Ok(ConnectionStatus::RateLimited)
                } else {
                    Ok(ConnectionStatus::Error(e.to_string()))
                }
            }
        }
    }
    
    async fn fetch_data(&self, params: Value) -> Result<ConnectorData> {
        // Construct the report request
        let mut report_request = json!({
            "dateRanges": [{
                "startDate": "30daysAgo",
                "endDate": "today"
            }]
        });
        
        // Override with date ranges from params if provided
        if let Some(date_ranges) = params.get("dateRanges") {
            report_request["dateRanges"] = date_ranges.clone();
        }
        
        // Add dimensions from params
        if let Some(dimensions) = params.get("dimensions") {
            report_request["dimensions"] = dimensions.clone();
        } else {
            return Err(anyhow!("Missing required parameter: dimensions"));
        }
        
        // Add metrics from params
        if let Some(metrics) = params.get("metrics") {
            report_request["metrics"] = metrics.clone();
        } else {
            return Err(anyhow!("Missing required parameter: metrics"));
        }
        
        // Add optional parameters
        if let Some(limit) = params.get("limit") {
            report_request["limit"] = limit.clone();
        }
        
        if let Some(offset) = params.get("offset") {
            report_request["offset"] = offset.clone();
        }
        
        if let Some(filters) = params.get("dimensionFilter") {
            report_request["dimensionFilter"] = filters.clone();
        }
        
        if let Some(filters) = params.get("metricFilter") {
            report_request["metricFilter"] = filters.clone();
        }
        
        // Set sampling level 
        let sampling = self.connection.sampling_level.as_deref().unwrap_or("DEFAULT");
        report_request["samplingLevel"] = json!(sampling);
        
        // Set currency if needed
        if self.connection.currency.is_some() {
            report_request["currencyCode"] = json!(self.connection.currency.as_ref().unwrap());
        }
        
        // Run the report
        let report_data = self.run_report(report_request.clone()).await?;
        
        // Transform the data
        let transformed_data = self.transform_report_data(report_data.clone())?;
        
        // Create metadata
        let mut metadata = HashMap::new();
        
        // Add information about sampling if present
        if let Some(is_sampled) = report_data["sampling"].as_object() {
            metadata.insert("is_sampled".to_string(), json!(true));
            if let Some(sample_size) = is_sampled.get("samplesReadCount") {
                metadata.insert("sample_size".to_string(), sample_size.clone());
            }
            if let Some(sample_space) = is_sampled.get("samplingSpaceSize") {
                metadata.insert("sample_space".to_string(), sample_space.clone());
            }
        } else {
            metadata.insert("is_sampled".to_string(), json!(false));
        }
        
        // Add row count
        if let Some(rows) = transformed_data.as_array() {
            metadata.insert("row_count".to_string(), json!(rows.len()));
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
        if !self.enabled {
            return Err(anyhow!("Connector is not enabled"));
        }
        
        // Ensure we have an access token
        let access_token = self.access_token.as_ref()
            .ok_or_else(|| anyhow!("Not authenticated"))?;
        
        // Get metadata about available dimensions and metrics
        let url = format!(
            "https://analyticsdata.googleapis.com/v1beta/properties/{}/metadata",
            self.connection.property_id
        );
        
        let response = self.client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("GA4 API error: {}", error_text));
        }
        
        let metadata: Value = response.json().await?;
        
        // Transform the metadata into a more usable format
        let mut dimensions = Vec::new();
        let mut metrics = Vec::new();
        
        if let Some(items) = metadata["dimensions"].as_array() {
            for item in items {
                if let Some(api_name) = item["apiName"].as_str() {
                    let category = item["category"].as_str().unwrap_or("UNCATEGORIZED");
                    let ui_name = item["uiName"].as_str().unwrap_or(api_name);
                    
                    dimensions.push(json!({
                        "name": api_name,
                        "display_name": ui_name,
                        "category": category
                    }));
                }
            }
        }
        
        if let Some(items) = metadata["metrics"].as_array() {
            for item in items {
                if let Some(api_name) = item["apiName"].as_str() {
                    let category = item["category"].as_str().unwrap_or("UNCATEGORIZED");
                    let ui_name = item["uiName"].as_str().unwrap_or(api_name);
                    let type_str = item["type"].as_str().unwrap_or("UNKNOWN");
                    
                    metrics.push(json!({
                        "name": api_name,
                        "display_name": ui_name,
                        "category": category,
                        "type": type_str
                    }));
                }
            }
        }
        
        Ok(json!({
            "property_id": self.connection.property_id,
            "dimensions": dimensions,
            "metrics": metrics
        }))
    }
    
    fn get_configuration_template(&self) -> Value {
        json!({
            "connection": {
                "property_id": {
                    "type": "string",
                    "required": true,
                    "description": "Google Analytics 4 Property ID (format: 123456789)"
                },
                "default_date_range_days": {
                    "type": "integer",
                    "required": false,
                    "default": 30,
                    "description": "Default date range in days (from today)"
                },
                "sampling_level": {
                    "type": "string",
                    "required": false,
                    "enum": ["DEFAULT", "SMALL", "LARGE"],
                    "default": "DEFAULT",
                    "description": "Data sampling level"
                },
                "currency": {
                    "type": "string",
                    "required": false,
                    "default": "USD",
                    "description": "Currency for monetary metrics (ISO currency code)"
                }
            },
            "auth": {
                "auth_type": {
                    "type": "string",
                    "required": true,
                    "enum": ["oauth", "service_account"],
                    "description": "Authentication method to use"
                },
                "client_id": {
                    "type": "string",
                    "required": true,
                    "condition": "auth_type === 'oauth'",
                    "description": "OAuth client ID"
                },
                "client_secret": {
                    "type": "string",
                    "required": true,
                    "condition": "auth_type === 'oauth'",
                    "description": "OAuth client secret"
                },
                "auth_code": {
                    "type": "string",
                    "required": false,
                    "condition": "auth_type === 'oauth' && !refresh_token",
                    "description": "OAuth authorization code (only needed for initial authentication)"
                },
                "refresh_token": {
                    "type": "string",
                    "required": false,
                    "condition": "auth_type === 'oauth'",
                    "description": "OAuth refresh token (if already authenticated)"
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
