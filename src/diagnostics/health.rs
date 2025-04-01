use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};
use chrono::{DateTime, Utc};

/// Status of a system component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusLevel {
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "unknown")]
    Unknown,
}

/// Status details for a system component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    /// Name of the component
    pub name: String,
    /// Current status level
    pub status: StatusLevel,
    /// Optional message describing the status
    pub message: Option<String>,
    /// Timestamp when the status was last checked
    pub last_checked: DateTime<Utc>,
    /// Additional details about the component
    pub details: HashMap<String, String>,
}

/// Overall system health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    /// Overall status of the system
    pub status: StatusLevel,
    /// Individual component statuses
    pub components: Vec<ComponentStatus>,
    /// Timestamp when the health check was performed
    pub timestamp: DateTime<Utc>,
}

impl SystemHealth {
    /// Create a new system health status
    pub fn new() -> Self {
        SystemHealth {
            status: StatusLevel::Unknown,
            components: Vec::new(),
            timestamp: Utc::now(),
        }
    }
    
    /// Add a component status
    pub fn add_component(&mut self, component: ComponentStatus) {
        // Update overall status based on component status
        self.status = match (&self.status, &component.status) {
            (_, StatusLevel::Error) => StatusLevel::Error,
            (StatusLevel::Ok, StatusLevel::Warning) => StatusLevel::Warning,
            (StatusLevel::Unknown, _) => component.status.clone(),
            _ => self.status.clone(),
        };
        
        self.components.push(component);
        self.timestamp = Utc::now();
    }
    
    /// Get a summary of the system health
    pub fn get_summary(&self) -> String {
        let total = self.components.len();
        let ok_count = self.components.iter().filter(|c| matches!(c.status, StatusLevel::Ok)).count();
        let warning_count = self.components.iter().filter(|c| matches!(c.status, StatusLevel::Warning)).count();
        let error_count = self.components.iter().filter(|c| matches!(c.status, StatusLevel::Error)).count();
        
        format!(
            "System Health: {:?} ({} OK, {} Warning, {} Error, {} total components)",
            self.status, ok_count, warning_count, error_count, total
        )
    }
}

/// Health checker for the system
pub struct HealthChecker {
    /// Database checker
    pub db_checker: Box<dyn DatabaseHealthChecker>,
    /// API checker
    pub api_checker: Box<dyn ApiHealthChecker>,
    /// Connector checkers by connector ID
    pub connector_checkers: HashMap<String, Box<dyn ConnectorHealthChecker>>,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(
        db_checker: Box<dyn DatabaseHealthChecker>,
        api_checker: Box<dyn ApiHealthChecker>,
    ) -> Self {
        HealthChecker {
            db_checker,
            api_checker,
            connector_checkers: HashMap::new(),
        }
    }
    
    /// Add a connector health checker
    pub fn add_connector_checker(&mut self, id: String, checker: Box<dyn ConnectorHealthChecker>) {
        self.connector_checkers.insert(id, checker);
    }
    
    /// Check health of the entire system
    pub async fn check_health(&self) -> SystemHealth {
        let mut health = SystemHealth::new();
        
        // Check database health
        match self.db_checker.check_health().await {
            Ok(status) => {
                info!("Database health check: {:?}", status.status);
                health.add_component(status);
            },
            Err(e) => {
                error!("Database health check failed: {}", e);
                health.add_component(ComponentStatus {
                    name: "database".to_string(),
                    status: StatusLevel::Error,
                    message: Some(format!("Health check failed: {}", e)),
                    last_checked: Utc::now(),
                    details: HashMap::new(),
                });
            }
        }
        
        // Check API health
        match self.api_checker.check_health().await {
            Ok(status) => {
                info!("API health check: {:?}", status.status);
                health.add_component(status);
            },
            Err(e) => {
                error!("API health check failed: {}", e);
                health.add_component(ComponentStatus {
                    name: "api".to_string(),
                    status: StatusLevel::Error,
                    message: Some(format!("Health check failed: {}", e)),
                    last_checked: Utc::now(),
                    details: HashMap::new(),
                });
            }
        }
        
        // Check connector health
        for (id, checker) in &self.connector_checkers {
            match checker.check_health().await {
                Ok(status) => {
                    info!("Connector {} health check: {:?}", id, status.status);
                    health.add_component(status);
                },
                Err(e) => {
                    error!("Connector {} health check failed: {}", id, e);
                    health.add_component(ComponentStatus {
                        name: format!("connector_{}", id),
                        status: StatusLevel::Error,
                        message: Some(format!("Health check failed: {}", e)),
                        last_checked: Utc::now(),
                        details: HashMap::new(),
                    });
                }
            }
        }
        
        info!("{}", health.get_summary());
        health
    }
}

/// Database health checker trait
#[async_trait::async_trait]
pub trait DatabaseHealthChecker: Send + Sync {
    /// Check database health
    async fn check_health(&self) -> Result<ComponentStatus>;
}

/// API health checker trait
#[async_trait::async_trait]
pub trait ApiHealthChecker: Send + Sync {
    /// Check API health
    async fn check_health(&self) -> Result<ComponentStatus>;
}

/// Connector health checker trait
#[async_trait::async_trait]
pub trait ConnectorHealthChecker: Send + Sync {
    /// Check connector health
    async fn check_health(&self) -> Result<ComponentStatus>;
}

/// SQLite database health checker
pub struct SqliteHealthChecker {
    /// Database connection pool
    pub pool: sqlx::SqlitePool,
}

#[async_trait::async_trait]
impl DatabaseHealthChecker for SqliteHealthChecker {
    async fn check_health(&self) -> Result<ComponentStatus> {
        let start = std::time::Instant::now();
        
        // Try to execute a simple query
        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => {
                let elapsed = start.elapsed();
                
                let mut details = HashMap::new();
                details.insert("response_time_ms".to_string(), elapsed.as_millis().to_string());
                details.insert("type".to_string(), "sqlite".to_string());
                
                Ok(ComponentStatus {
                    name: "database".to_string(),
                    status: StatusLevel::Ok,
                    message: Some("Database is responsive".to_string()),
                    last_checked: Utc::now(),
                    details,
                })
            },
            Err(e) => {
                let mut details = HashMap::new();
                details.insert("error".to_string(), e.to_string());
                details.insert("type".to_string(), "sqlite".to_string());
                
                Ok(ComponentStatus {
                    name: "database".to_string(),
                    status: StatusLevel::Error,
                    message: Some("Database query failed".to_string()),
                    last_checked: Utc::now(),
                    details,
                })
            }
        }
    }
}
