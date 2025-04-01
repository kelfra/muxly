use crate::connectors::base::{ConnectorData, ConnectorSettings, ConnectionStatus, Connector};
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use std::collections::HashMap;
use reqwest::{Client, Method, header};
use chrono::Utc;
use std::str::FromStr;

/// API connector for custom API endpoints
pub struct APIConnector {
    /// Client ID for the connector
    id: String,
    /// Human-readable name for the connector
    name: String,
    /// Whether this connector is enabled
    enabled: bool,
    /// HTTP client for making requests
    client: Client,
    /// Base URL for the API
    base_url: String,
    /// HTTP method for the request
    method: Method,
    /// Headers to include in the request
    headers: HashMap<String, String>,
    /// Query parameters
    query_params: HashMap<String, String>,
    /// Body template (for POST/PUT/PATCH)
    body_template: Option<Value>,
    /// Path parameters
    path_params: HashMap<String, String>,
    /// Authentication token, if any
    auth_token: Option<String>,
    /// Authentication type (bearer, basic, api_key)
    auth_type: Option<String>,
    /// JSON path to extract metrics from response
    metrics_path: Option<String>,
}

impl APIConnector {
    /// Create a new APIConnector instance
    pub fn new() -> Self {
        APIConnector {
            id: String::new(),
            name: String::new(),
            enabled: false,
            client: Client::new(),
            base_url: String::new(),
            method: Method::GET,
            headers: HashMap::new(),
            query_params: HashMap::new(),
            body_template: None,
            path_params: HashMap::new(),
            auth_token: None,
            auth_type: None,
            metrics_path: None,
        }
    }

    /// Replace path parameters in URL
    fn replace_path_params(&self, url: &str) -> String {
        let mut result = url.to_string();
        for (key, value) in &self.path_params {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }

    /// Extract metrics from response using JSON path
    fn extract_metrics(&self, response: &Value) -> Result<Value> {
        if let Some(path) = &self.metrics_path {
            let mut current = response;
            for part in path.split('.') {
                if let Some(index) = part.strip_suffix(']').and_then(|s| s.strip_prefix('[')) {
                    if let Ok(idx) = index.parse::<usize>() {
                        if let Some(array) = current.as_array() {
                            if idx < array.len() {
                                current = &array[idx];
                            } else {
                                return Err(anyhow!("Array index out of bounds: {}", idx));
                            }
                        } else {
                            return Err(anyhow!("Expected array but found: {}", current));
                        }
                    } else {
                        return Err(anyhow!("Invalid array index: {}", index));
                    }
                } else {
                    if let Some(obj) = current.as_object() {
                        if let Some(val) = obj.get(part) {
                            current = val;
                        } else {
                            return Err(anyhow!("Key not found in object: {}", part));
                        }
                    } else {
                        return Err(anyhow!("Expected object but found: {}", current));
                    }
                }
            }
            Ok(current.clone())
        } else {
            Ok(response.clone())
        }
    }
}

#[async_trait]
impl Connector for APIConnector {
    fn get_type(&self) -> &str {
        "api"
    }

    fn get_id(&self) -> &str {
        &self.id
    }

    async fn initialize(&mut self, settings: ConnectorSettings) -> Result<()> {
        self.id = settings.id;
        self.name = settings.name;
        self.enabled = settings.enabled;

        // Get connection settings
        if let Some(base_url) = settings.connection.get("base_url") {
            self.base_url = base_url.as_str()
                .ok_or_else(|| anyhow!("base_url must be a string"))?
                .to_string();
        } else {
            return Err(anyhow!("Missing required setting: base_url"));
        }

        // Get HTTP method
        if let Some(method) = settings.connection.get("method") {
            let method_str = method.as_str()
                .ok_or_else(|| anyhow!("method must be a string"))?;
            self.method = Method::from_str(method_str)
                .map_err(|_| anyhow!("Invalid HTTP method: {}", method_str))?;
        }

        // Get headers
        if let Some(headers) = settings.connection.get("headers") {
            if let Some(headers_obj) = headers.as_object() {
                for (key, value) in headers_obj {
                    if let Some(value_str) = value.as_str() {
                        self.headers.insert(key.clone(), value_str.to_string());
                    }
                }
            }
        }

        // Get query parameters
        if let Some(params) = settings.connection.get("query_params") {
            if let Some(params_obj) = params.as_object() {
                for (key, value) in params_obj {
                    if let Some(value_str) = value.as_str() {
                        self.query_params.insert(key.clone(), value_str.to_string());
                    }
                }
            }
        }

        // Get body template
        if let Some(body) = settings.connection.get("body_template") {
            self.body_template = Some(body.clone());
        }

        // Get path parameters
        if let Some(path_params) = settings.connection.get("path_params") {
            if let Some(params_obj) = path_params.as_object() {
                for (key, value) in params_obj {
                    if let Some(value_str) = value.as_str() {
                        self.path_params.insert(key.clone(), value_str.to_string());
                    }
                }
            }
        }

        // Get metrics path
        if let Some(metrics_path) = settings.connection.get("metrics_path") {
            self.metrics_path = metrics_path.as_str().map(|s| s.to_string());
        }

        // Handle authentication
        match settings.auth.auth_type.as_str() {
            "bearer" => {
                self.auth_type = Some("bearer".to_string());
                if let Some(token) = settings.auth.params.get("token") {
                    self.auth_token = token.as_str().map(|s| s.to_string());
                } else {
                    return Err(anyhow!("Missing token for bearer authentication"));
                }
            },
            "basic" => {
                self.auth_type = Some("basic".to_string());
                let username = settings.auth.params.get("username")
                    .and_then(|u| u.as_str())
                    .ok_or_else(|| anyhow!("Missing username for basic authentication"))?;
                let password = settings.auth.params.get("password")
                    .and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow!("Missing password for basic authentication"))?;
                
                let token = base64::encode(format!("{}:{}", username, password));
                self.auth_token = Some(token);
            },
            "api_key" => {
                // API Key can be in header, query param, or in the URL
                let api_key = settings.auth.params.get("api_key")
                    .and_then(|k| k.as_str())
                    .ok_or_else(|| anyhow!("Missing api_key for API key authentication"))?;
                
                let location = settings.auth.params.get("location")
                    .and_then(|l| l.as_str())
                    .unwrap_or("header");
                
                match location {
                    "header" => {
                        let header_name = settings.auth.params.get("header_name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("X-API-Key");
                        self.headers.insert(header_name.to_string(), api_key.to_string());
                    },
                    "query" => {
                        let param_name = settings.auth.params.get("param_name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("api_key");
                        self.query_params.insert(param_name.to_string(), api_key.to_string());
                    },
                    _ => return Err(anyhow!("Unsupported API key location: {}", location)),
                }
            },
            "none" => {
                // No authentication required
            },
            _ => return Err(anyhow!("Unsupported authentication type: {}", settings.auth.auth_type)),
        }

        Ok(())
    }

    async fn test_connection(&self) -> Result<ConnectionStatus> {
        if !self.enabled {
            return Ok(ConnectionStatus::Disconnected);
        }
        
        let url = self.replace_path_params(&self.base_url);
        let mut request = self.client.request(Method::GET, &url);
        
        // Add query parameters
        request = request.query(&self.query_params);
        
        // Add headers
        let mut headers = header::HeaderMap::new();
        for (key, value) in &self.headers {
            if let Ok(header_name) = header::HeaderName::from_str(key) {
                if let Ok(header_value) = header::HeaderValue::from_str(value) {
                    headers.insert(header_name, header_value);
                }
            }
        }
        
        // Add authentication
        if let (Some(auth_type), Some(token)) = (&self.auth_type, &self.auth_token) {
            match auth_type.as_str() {
                "bearer" => {
                    headers.insert(
                        header::AUTHORIZATION,
                        header::HeaderValue::from_str(&format!("Bearer {}", token))
                            .map_err(|e| anyhow!("Invalid authorization header: {}", e))?,
                    );
                },
                "basic" => {
                    headers.insert(
                        header::AUTHORIZATION,
                        header::HeaderValue::from_str(&format!("Basic {}", token))
                            .map_err(|e| anyhow!("Invalid authorization header: {}", e))?,
                    );
                },
                _ => {}
            }
        }
        
        request = request.headers(headers);
        
        // Send request
        match request.send().await {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    Ok(ConnectionStatus::Connected)
                } else if status.as_u16() == 401 || status.as_u16() == 403 {
                    Ok(ConnectionStatus::CredentialsInvalid)
                } else if status.as_u16() == 429 {
                    Ok(ConnectionStatus::RateLimited)
                } else {
                    Ok(ConnectionStatus::Error(format!("HTTP error: {}", status)))
                }
            },
            Err(e) => Ok(ConnectionStatus::Error(format!("Connection error: {}", e))),
        }
    }

    async fn fetch_data(&self, params: Value) -> Result<ConnectorData> {
        if !self.enabled {
            return Err(anyhow!("Connector is disabled"));
        }
        
        let mut url = self.replace_path_params(&self.base_url);
        
        // Add dynamic path parameters from params if any
        if let Some(dynamic_path_params) = params.get("path_params") {
            if let Some(path_params_obj) = dynamic_path_params.as_object() {
                for (key, value) in path_params_obj {
                    if let Some(value_str) = value.as_str() {
                        let placeholder = format!("{{{}}}", key);
                        url = url.replace(&placeholder, value_str);
                    }
                }
            }
        }
        
        // Start building the request
        let mut request = self.client.request(self.method.clone(), &url);
        
        // Merge static and dynamic query parameters
        let mut query_params = self.query_params.clone();
        if let Some(dynamic_query_params) = params.get("query_params") {
            if let Some(query_params_obj) = dynamic_query_params.as_object() {
                for (key, value) in query_params_obj {
                    if let Some(value_str) = value.as_str() {
                        query_params.insert(key.clone(), value_str.to_string());
                    }
                }
            }
        }
        request = request.query(&query_params);
        
        // Add headers
        let mut headers = header::HeaderMap::new();
        for (key, value) in &self.headers {
            if let Ok(header_name) = header::HeaderName::from_str(key) {
                if let Ok(header_value) = header::HeaderValue::from_str(value) {
                    headers.insert(header_name, header_value);
                }
            }
        }
        
        // Add dynamic headers
        if let Some(dynamic_headers) = params.get("headers") {
            if let Some(headers_obj) = dynamic_headers.as_object() {
                for (key, value) in headers_obj {
                    if let Some(value_str) = value.as_str() {
                        if let Ok(header_name) = header::HeaderName::from_str(key) {
                            if let Ok(header_value) = header::HeaderValue::from_str(value_str) {
                                headers.insert(header_name, header_value);
                            }
                        }
                    }
                }
            }
        }
        
        // Add authentication
        if let (Some(auth_type), Some(token)) = (&self.auth_type, &self.auth_token) {
            match auth_type.as_str() {
                "bearer" => {
                    headers.insert(
                        header::AUTHORIZATION,
                        header::HeaderValue::from_str(&format!("Bearer {}", token))
                            .map_err(|e| anyhow!("Invalid authorization header: {}", e))?,
                    );
                },
                "basic" => {
                    headers.insert(
                        header::AUTHORIZATION,
                        header::HeaderValue::from_str(&format!("Basic {}", token))
                            .map_err(|e| anyhow!("Invalid authorization header: {}", e))?,
                    );
                },
                _ => {}
            }
        }
        
        request = request.headers(headers);
        
        // Add body if needed for POST/PUT/PATCH
        if self.method != Method::GET && self.method != Method::HEAD {
            let mut body = self.body_template.clone().unwrap_or(json!({}));
            
            // Merge dynamic body parameters if any
            if let Some(dynamic_body) = params.get("body") {
                if let (Some(body_obj), Some(dynamic_obj)) = (body.as_object_mut(), dynamic_obj.as_object()) {
                    for (key, value) in dynamic_obj {
                        body_obj.insert(key.clone(), value.clone());
                    }
                }
            }
            
            request = request.json(&body);
        }
        
        // Send request
        let response = request.send().await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;
        
        // Check status
        let status = response.status();
        if !status.is_success() {
            return Err(anyhow!("API request failed with status: {}", status));
        }
        
        // Parse JSON response
        let response_json: Value = response.json().await
            .map_err(|e| anyhow!("Failed to parse response as JSON: {}", e))?;
        
        // Extract metrics if path is specified
        let metrics = self.extract_metrics(&response_json)?;
        
        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("url".to_string(), json!(url));
        metadata.insert("status_code".to_string(), json!(status.as_u16()));
        metadata.insert("method".to_string(), json!(self.method.as_str()));
        metadata.insert("timestamp".to_string(), json!(Utc::now().to_rfc3339()));
        
        // Return the connector data
        Ok(ConnectorData {
            connector_id: self.id.clone(),
            timestamp: Utc::now(),
            data: metrics,
            metadata,
        })
    }

    async fn get_metadata(&self) -> Result<Value> {
        Ok(json!({
            "connector_type": "api",
            "base_url": self.base_url,
            "method": self.method.as_str(),
            "headers": self.headers,
            "query_params": self.query_params,
            "auth_type": self.auth_type,
            "metrics_path": self.metrics_path,
        }))
    }

    fn get_configuration_template(&self) -> Value {
        json!({
            "connector": {
                "id": "custom-api",
                "name": "Custom API Connector",
                "connector_type": "api",
                "enabled": true,
                "auth": {
                    "auth_type": "bearer",
                    "params": {
                        "token": "your-token-here"
                    }
                },
                "connection": {
                    "base_url": "https://api.example.com/metrics/{endpoint}",
                    "method": "GET",
                    "headers": {
                        "Content-Type": "application/json",
                        "Accept": "application/json"
                    },
                    "query_params": {
                        "start_date": "2023-01-01",
                        "end_date": "2023-01-31"
                    },
                    "path_params": {
                        "endpoint": "users"
                    },
                    "metrics_path": "data.metrics",
                    "body_template": {
                        "filters": {
                            "status": "active"
                        }
                    }
                },
                "rate_limit": {
                    "max_requests": 100,
                    "period_seconds": 60,
                    "auto_adjust": true
                },
                "retry": {
                    "max_attempts": 3,
                    "initial_backoff_ms": 1000,
                    "max_backoff_ms": 30000,
                    "backoff_multiplier": 2.0,
                    "retryable_errors": ["RATE_LIMIT", "SERVER_ERROR"]
                }
            }
        })
    }
} 