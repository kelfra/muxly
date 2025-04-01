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

/// HubSpot-specific connection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubSpotConnectionSettings {
    /// API version to use
    pub api_version: Option<String>,
    /// Maximum number of records to fetch per request
    pub batch_size: Option<u32>,
    /// Whether to include archived records
    pub include_archived: Option<bool>,
    /// Properties to fetch for each object type
    pub properties: Option<HashMap<String, Vec<String>>>,
}

/// HubSpot API object types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HubSpotObjectType {
    Contacts,
    Companies,
    Deals,
    Tickets,
    Products,
    LineItems,
    Custom,
}

impl HubSpotObjectType {
    /// Convert object type to API path
    fn to_path(&self) -> &'static str {
        match self {
            Self::Contacts => "contacts",
            Self::Companies => "companies",
            Self::Deals => "deals",
            Self::Tickets => "tickets",
            Self::Products => "products",
            Self::LineItems => "line_items",
            Self::Custom => "custom",
        }
    }
    
    /// Convert from string
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "contacts" => Ok(Self::Contacts),
            "companies" => Ok(Self::Companies),
            "deals" => Ok(Self::Deals),
            "tickets" => Ok(Self::Tickets),
            "products" => Ok(Self::Products),
            "line_items" | "lineitems" => Ok(Self::LineItems),
            "custom" => Ok(Self::Custom),
            _ => Err(anyhow!("Unknown HubSpot object type: {}", s)),
        }
    }
}

/// HubSpot Connector implementation
pub struct HubSpotConnector {
    /// Connector identifier
    id: String,
    /// Human-readable name
    name: String,
    /// Whether this connector is enabled
    enabled: bool,
    /// Authentication settings
    auth: AuthSettings,
    /// Connection settings specific to HubSpot
    connection: HubSpotConnectionSettings,
    /// HTTP client for API requests
    client: Client,
    /// Authentication token
    access_token: Option<String>,
    /// Token expiration time
    token_expiry: Option<chrono::DateTime<chrono::Utc>>,
    /// Refresh token for OAuth
    refresh_token: Option<String>,
}

impl HubSpotConnector {
    /// Create a new HubSpot connector
    pub fn new() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            enabled: false,
            auth: AuthSettings {
                auth_type: String::new(),
                params: HashMap::new(),
            },
            connection: HubSpotConnectionSettings {
                api_version: Some("v3".to_string()),
                batch_size: Some(100),
                include_archived: Some(false),
                properties: None,
            },
            client: Client::new(),
            access_token: None,
            token_expiry: None,
            refresh_token: None,
        }
    }

    /// Authenticate with HubSpot API
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
                let token_url = "https://api.hubapi.com/oauth/v1/token";
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
            "api_key" => {
                // Get the API key from the params
                let api_key = self.auth.params.get("api_key")
                    .ok_or_else(|| anyhow!("Missing api_key parameter"))?
                    .as_str()
                    .ok_or_else(|| anyhow!("api_key must be a string"))?;
                
                // With API key auth, we don't have refresh tokens or expiration
                self.access_token = Some(api_key.to_string());
                self.token_expiry = None; // API keys don't expire
                
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

        let token_url = "https://api.hubapi.com/oauth/v1/token";
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

    /// Fetch objects from HubSpot
    async fn fetch_objects(&self, object_type: HubSpotObjectType, params: &Value) -> Result<Value> {
        // Ensure we have an access token
        let access_token = self.access_token.as_ref()
            .ok_or_else(|| anyhow!("Not authenticated"))?;
        
        // Determine which API version to use
        let api_version = self.connection.api_version.as_deref().unwrap_or("v3");

        // Base URL for the CRM API
        let base_url = format!("https://api.hubapi.com/crm/{}/objects/{}", api_version, object_type.to_path());
        
        // Get optional parameters
        let batch_size = params.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(self.connection.batch_size.unwrap_or(100) as u64);
        
        let include_archived = params.get("includeArchived")
            .and_then(|v| v.as_bool())
            .unwrap_or(self.connection.include_archived.unwrap_or(false));
        
        // Get properties to fetch, falling back to configured properties if not specified
        let properties = if let Some(props) = params.get("properties") {
            if let Some(props_array) = props.as_array() {
                props_array.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
            } else {
                return Err(anyhow!("properties must be an array"));
            }
        } else if let Some(config_props) = &self.connection.properties {
            // Use properties from connection config if defined for this object type
            config_props.get(object_type.to_path())
                .map(|v| v.iter().map(|s| s.as_str()).collect::<Vec<_>>())
                .unwrap_or_else(|| vec![])
        } else {
            // No properties specified
            vec![]
        };
        
        // Build the API URL with query parameters
        let mut url = format!("{}?limit={}", base_url, batch_size);
        
        if include_archived {
            url.push_str("&archived=true");
        }
        
        for prop in &properties {
            url.push_str(&format!("&properties={}", prop));
        }
        
        // Add search criteria if provided
        if let Some(search) = params.get("search") {
            if let Some(search_str) = search.as_str() {
                url.push_str(&format!("&search={}", search_str));
            }
        }
        
        // Add filter if provided
        if let Some(filter) = params.get("filter") {
            if let Some(filter_obj) = filter.as_object() {
                // For simplicity we're just converting the filter object to a string
                // In a real implementation, we would build a proper filter structure
                let filter_json = serde_json::to_string(filter_obj)?;
                url.push_str(&format!("&filter={}", filter_json));
            }
        }
        
        // Add sorting if provided
        if let Some(sort) = params.get("sort") {
            if let Some(sort_array) = sort.as_array() {
                for sort_item in sort_array {
                    if let Some(sort_obj) = sort_item.as_object() {
                        if let (Some(property), Some(direction)) = (
                            sort_obj.get("property").and_then(|v| v.as_str()),
                            sort_obj.get("direction").and_then(|v| v.as_str())
                        ) {
                            url.push_str(&format!("&sort={},{}", property, direction));
                        }
                    }
                }
            }
        }
        
        // Add pagination cursor if provided
        if let Some(after) = params.get("after").and_then(|v| v.as_str()) {
            url.push_str(&format!("&after={}", after));
        }
        
        // Make the API request
        let response = self.client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        // Check for rate limiting
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            // Extract retry-after header if present
            let retry_after = response.headers()
                .get(header::RETRY_AFTER)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(10); // Default to 10 seconds

            // Wait and retry
            sleep(Duration::from_secs(retry_after)).await;
            return self.fetch_objects(object_type, params).await;
        }

        // Check for errors
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("HubSpot API error: {}", error_text));
        }

        // Parse the response
        let data: Value = response.json().await?;
        Ok(data)
    }

    /// Fetch associations between objects
    async fn fetch_associations(&self, object_type: HubSpotObjectType, object_id: &str, to_object_type: HubSpotObjectType, params: &Value) -> Result<Value> {
        // Ensure we have an access token
        let access_token = self.access_token.as_ref()
            .ok_or_else(|| anyhow!("Not authenticated"))?;
        
        // Determine which API version to use
        let api_version = self.connection.api_version.as_deref().unwrap_or("v3");

        // Build the API URL for associations
        let url = format!(
            "https://api.hubapi.com/crm/{}/objects/{}/{}/associations/{}",
            api_version,
            object_type.to_path(),
            object_id,
            to_object_type.to_path()
        );
        
        // Make the API request
        let response = self.client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        // Check for errors
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("HubSpot API error: {}", error_text));
        }

        // Parse the response
        let data: Value = response.json().await?;
        Ok(data)
    }
}

#[async_trait]
impl Connector for HubSpotConnector {
    fn get_type(&self) -> &str {
        "hubspot"
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
        
        // Simple test request to check connection - fetch single contact with minimal properties
        let test_params = json!({
            "limit": 1,
            "properties": ["firstname", "lastname"]
        });
        
        match self.fetch_objects(HubSpotObjectType::Contacts, &test_params).await {
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
        // Extract object type from params
        let object_type_str = params.get("objectType")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: objectType"))?;
        
        let object_type = HubSpotObjectType::from_str(object_type_str)?;
        
        // Check if this is an association request
        let is_association = params.get("associationType").is_some() && params.get("objectId").is_some();
        
        let result = if is_association {
            // This is an association request
            let object_id = params["objectId"]
                .as_str()
                .ok_or_else(|| anyhow!("Missing required parameter: objectId"))?;
                
            let to_object_type_str = params["associationType"]
                .as_str()
                .ok_or_else(|| anyhow!("Missing required parameter: associationType"))?;
                
            let to_object_type = HubSpotObjectType::from_str(to_object_type_str)?;
            
            self.fetch_associations(object_type, object_id, to_object_type, &params).await?
        } else {
            // This is a standard object fetch request
            self.fetch_objects(object_type, &params).await?
        };
        
        // Extract record results
        let results = if is_association {
            // Association response structure is different
            result["results"].clone()
        } else {
            // Get the results array from the response
            let mut transformed_data = Vec::new();
            
            if let Some(results) = result["results"].as_array() {
                for item in results {
                    // Extract properties and add id
                    let mut record = serde_json::Map::new();
                    
                    // Add id
                    if let Some(id) = item["id"].as_str() {
                        record.insert("id".to_string(), json!(id));
                    }
                    
                    // Add created and updated timestamps
                    if let Some(created) = item["createdAt"].as_str() {
                        record.insert("createdAt".to_string(), json!(created));
                    }
                    
                    if let Some(updated) = item["updatedAt"].as_str() {
                        record.insert("updatedAt".to_string(), json!(updated));
                    }
                    
                    // Add properties
                    if let Some(props) = item["properties"].as_object() {
                        for (key, value) in props {
                            record.insert(key.clone(), value.clone());
                        }
                    }
                    
                    transformed_data.push(Value::Object(record));
                }
            }
            
            json!(transformed_data)
        };
        
        // Create metadata
        let mut metadata = HashMap::new();
        
        // Add pagination info
        if let Some(pagination) = result.get("paging") {
            metadata.insert("pagination".to_string(), pagination.clone());
        }
        
        // Add total records count if available
        if let Some(total) = result.get("total") {
            metadata.insert("total".to_string(), total.clone());
        }
        
        // Return the connector data
        Ok(ConnectorData {
            connector_id: self.id.clone(),
            timestamp: Utc::now(),
            data: results,
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
        
        // Get metadata about available object schemas
        let api_version = self.connection.api_version.as_deref().unwrap_or("v3");
        let url = format!(
            "https://api.hubapi.com/crm/{}/schemas",
            api_version
        );
        
        let response = self.client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("HubSpot API error: {}", error_text));
        }
        
        let schemas: Value = response.json().await?;
        
        // Transform the schemas into a more usable format
        let mut object_types = Vec::new();
        
        if let Some(items) = schemas["results"].as_array() {
            for item in items {
                if let (Some(name), Some(label)) = (
                    item["name"].as_str(),
                    item["labels"]["singular"].as_str()
                ) {
                    // Extract properties
                    let mut properties = Vec::new();
                    
                    if let Some(props) = item["properties"].as_array() {
                        for prop in props {
                            if let (Some(prop_name), Some(prop_label)) = (
                                prop["name"].as_str(),
                                prop["label"].as_str()
                            ) {
                                let prop_type = prop["type"].as_str().unwrap_or("string");
                                properties.push(json!({
                                    "name": prop_name,
                                    "label": prop_label,
                                    "type": prop_type,
                                    "required": prop["required"].as_bool().unwrap_or(false)
                                }));
                            }
                        }
                    }
                    
                    object_types.push(json!({
                        "name": name,
                        "label": label,
                        "properties": properties
                    }));
                }
            }
        }
        
        Ok(json!({
            "object_types": object_types
        }))
    }
    
    fn get_configuration_template(&self) -> Value {
        json!({
            "connection": {
                "api_version": {
                    "type": "string",
                    "required": false,
                    "default": "v3",
                    "description": "HubSpot API version to use"
                },
                "batch_size": {
                    "type": "integer",
                    "required": false,
                    "default": 100,
                    "description": "Maximum number of records to fetch per request"
                },
                "include_archived": {
                    "type": "boolean",
                    "required": false,
                    "default": false,
                    "description": "Whether to include archived records in results"
                },
                "properties": {
                    "type": "object",
                    "required": false,
                    "description": "Default properties to fetch for each object type",
                    "example": {
                        "contacts": ["firstname", "lastname", "email"],
                        "companies": ["name", "domain", "industry"]
                    }
                }
            },
            "auth": {
                "auth_type": {
                    "type": "string",
                    "required": true,
                    "enum": ["oauth", "api_key"],
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
                "api_key": {
                    "type": "string",
                    "required": true,
                    "condition": "auth_type === 'api_key'",
                    "description": "HubSpot API key"
                }
            }
        })
    }
}
