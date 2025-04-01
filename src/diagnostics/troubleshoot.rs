use anyhow::{Result, anyhow};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, warn, error};

use crate::connectors::base::{ConnectorSettings, ConnectionStatus};
use crate::diagnostics::connection::ConnectionTester;

/// Troubleshooting suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Title of the suggestion
    pub title: String,
    /// Description of the suggestion
    pub description: String,
    /// Priority of the suggestion (1-5, 1 being highest)
    pub priority: u8,
    /// Whether this is a quick fix
    pub quick_fix: bool,
}

/// Troubleshooting report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TroubleshootingReport {
    /// ID of the component being troubleshot
    pub component_id: String,
    /// Type of component
    pub component_type: String,
    /// Primary issue identified
    pub primary_issue: Option<String>,
    /// List of suggestions to fix the issue
    pub suggestions: Vec<Suggestion>,
    /// Detailed diagnostic information
    pub diagnostics: HashMap<String, Value>,
}

/// Troubleshooter for diagnosing issues
pub struct Troubleshooter {
    /// Connection tester for checking connectivity
    pub connection_tester: ConnectionTester,
}

impl Troubleshooter {
    /// Create a new troubleshooter
    pub fn new() -> Self {
        Troubleshooter {
            connection_tester: ConnectionTester::new(),
        }
    }
    
    /// Troubleshoot a connector
    pub async fn troubleshoot_connector(&self, settings: &ConnectorSettings) -> Result<TroubleshootingReport> {
        info!("Troubleshooting connector: {}", settings.id);
        
        let mut report = TroubleshootingReport {
            component_id: settings.id.clone(),
            component_type: settings.connector_type.clone(),
            primary_issue: None,
            suggestions: Vec::new(),
            diagnostics: HashMap::new(),
        };
        
        // Test the connection
        let connection_result = self.connection_tester.test_connection(settings).await?;
        
        // Add connection test results to diagnostics
        report.diagnostics.insert(
            "connection_test".to_string(),
            serde_json::to_value(&connection_result)?
        );
        
        // Diagnose based on connection status
        match &connection_result.status {
            ConnectionStatus::Connected => {
                // Connection is good, no primary issue
                report.primary_issue = None;
            },
            ConnectionStatus::Disconnected => {
                report.primary_issue = Some("Connection failed".to_string());
                report.suggestions.push(Suggestion {
                    title: "Check network connectivity".to_string(),
                    description: "Ensure that the service has network access to the target API or service.".to_string(),
                    priority: 1,
                    quick_fix: true,
                });
                report.suggestions.push(Suggestion {
                    title: "Verify service is running".to_string(),
                    description: "Check that the target service or API is running and accessible.".to_string(),
                    priority: 1,
                    quick_fix: true,
                });
            },
            ConnectionStatus::Error(msg) => {
                report.primary_issue = Some(format!("Connection error: {}", msg));
                
                // Add suggestions based on the error message
                if msg.contains("timeout") {
                    report.suggestions.push(Suggestion {
                        title: "Increase timeout setting".to_string(),
                        description: "The connection is timing out. Try increasing the timeout_seconds value in your configuration.".to_string(),
                        priority: 2,
                        quick_fix: true,
                    });
                }
                
                if msg.contains("DNS") || msg.contains("not found") {
                    report.suggestions.push(Suggestion {
                        title: "Check hostname".to_string(),
                        description: "The hostname may be incorrect. Verify the base_url setting in your configuration.".to_string(),
                        priority: 1,
                        quick_fix: true,
                    });
                }
                
                // Add a general suggestion
                report.suggestions.push(Suggestion {
                    title: "Check error logs".to_string(),
                    description: format!("Review the error details: {}", msg),
                    priority: 3,
                    quick_fix: false,
                });
            },
            ConnectionStatus::CredentialsInvalid => {
                report.primary_issue = Some("Invalid credentials".to_string());
                
                match settings.connector_type.as_str() {
                    "bigquery" => {
                        report.suggestions.push(Suggestion {
                            title: "Check BigQuery credentials".to_string(),
                            description: "Verify that your service account credentials are correct and have the necessary permissions.".to_string(),
                            priority: 1,
                            quick_fix: true,
                        });
                    },
                    "ga4" => {
                        report.suggestions.push(Suggestion {
                            title: "Check GA4 OAuth credentials".to_string(),
                            description: "Verify your client_id, client_secret, and make sure your access_token or refresh_token is valid.".to_string(),
                            priority: 1,
                            quick_fix: true,
                        });
                        report.suggestions.push(Suggestion {
                            title: "Refresh OAuth token".to_string(),
                            description: "Your OAuth token may have expired. Try refreshing it using the refresh_token.".to_string(),
                            priority: 2,
                            quick_fix: true,
                        });
                    },
                    "hubspot" => {
                        if settings.auth.auth_type == "api_key" {
                            report.suggestions.push(Suggestion {
                                title: "Check HubSpot API key".to_string(),
                                description: "Verify that your API key is correct and has the necessary permissions.".to_string(),
                                priority: 1,
                                quick_fix: true,
                            });
                        } else {
                            report.suggestions.push(Suggestion {
                                title: "Check HubSpot OAuth credentials".to_string(),
                                description: "Verify your client_id, client_secret, and make sure your access_token or refresh_token is valid.".to_string(),
                                priority: 1,
                                quick_fix: true,
                            });
                        }
                    },
                    _ => {
                        report.suggestions.push(Suggestion {
                            title: "Check authentication settings".to_string(),
                            description: "Verify that your authentication credentials are correct.".to_string(),
                            priority: 1,
                            quick_fix: true,
                        });
                    }
                }
            },
            ConnectionStatus::RateLimited => {
                report.primary_issue = Some("Rate limited by API".to_string());
                report.suggestions.push(Suggestion {
                    title: "Adjust rate limit settings".to_string(),
                    description: "The API is rate limiting requests. Try reducing the frequency of requests or implementing backoff.".to_string(),
                    priority: 2,
                    quick_fix: true,
                });
                report.suggestions.push(Suggestion {
                    title: "Check API quotas".to_string(),
                    description: "You may have exceeded your API quota. Check your API usage in the service provider's dashboard.".to_string(),
                    priority: 3,
                    quick_fix: false,
                });
            },
            ConnectionStatus::Unknown => {
                report.primary_issue = Some("Unknown connection status".to_string());
                report.suggestions.push(Suggestion {
                    title: "Check connection settings".to_string(),
                    description: "Verify all connection settings and try again.".to_string(),
                    priority: 3,
                    quick_fix: true,
                });
            },
        }
        
        // Check other potential issues
        self.check_scheduler_issues(settings, &mut report);
        self.check_configuration_issues(settings, &mut report);
        
        Ok(report)
    }
    
    /// Check for scheduler-related issues
    fn check_scheduler_issues(&self, settings: &ConnectorSettings, report: &mut TroubleshootingReport) {
        // In a real implementation, this would check the scheduler settings
        // For now, we'll just add a placeholder suggestion
        if !settings.enabled {
            report.suggestions.push(Suggestion {
                title: "Connector is disabled".to_string(),
                description: "This connector is currently disabled. Set 'enabled = true' to enable it.".to_string(),
                priority: 1,
                quick_fix: true,
            });
        }
    }
    
    /// Check for configuration-related issues
    fn check_configuration_issues(&self, settings: &ConnectorSettings, report: &mut TroubleshootingReport) {
        // In a real implementation, this would validate the configuration more thoroughly
        // For now, we'll just add a few generic checks
        
        // Check for empty connection parameters
        for (key, value) in &settings.connection {
            if let Some(value_str) = value.as_str() {
                if value_str.is_empty() {
                    report.suggestions.push(Suggestion {
                        title: format!("Empty configuration value: {}", key),
                        description: format!("The connection parameter '{}' is empty. Please provide a value.", key),
                        priority: 2,
                        quick_fix: true,
                    });
                }
            }
        }
        
        // Add connector-specific checks
        match settings.connector_type.as_str() {
            "bigquery" => {
                if !settings.connection.contains_key("project_id") {
                    report.suggestions.push(Suggestion {
                        title: "Missing project_id".to_string(),
                        description: "BigQuery connector requires a project_id in the connection settings.".to_string(),
                        priority: 1,
                        quick_fix: true,
                    });
                }
            },
            "ga4" => {
                if !settings.connection.contains_key("property_id") {
                    report.suggestions.push(Suggestion {
                        title: "Missing property_id".to_string(),
                        description: "GA4 connector requires a property_id in the connection settings.".to_string(),
                        priority: 1,
                        quick_fix: true,
                    });
                }
            },
            "hubspot" => {
                // No specific checks for now
            },
            "internal_api" => {
                if !settings.connection.contains_key("base_url") {
                    report.suggestions.push(Suggestion {
                        title: "Missing base_url".to_string(),
                        description: "Internal API connector requires a base_url in the connection settings.".to_string(),
                        priority: 1,
                        quick_fix: true,
                    });
                }
            },
            _ => {}
        }
    }
    
    /// Generate a human-readable troubleshooting report
    pub fn generate_report(&self, report: &TroubleshootingReport) -> String {
        let mut output = format!("Troubleshooting Report for {} ({})\n\n", 
            report.component_id, report.component_type);
        
        if let Some(issue) = &report.primary_issue {
            output.push_str(&format!("Primary Issue: {}\n\n", issue));
        } else {
            output.push_str("No issues detected!\n\n");
        }
        
        if !report.suggestions.is_empty() {
            output.push_str("Suggestions:\n");
            
            // Sort suggestions by priority
            let mut sorted_suggestions = report.suggestions.clone();
            sorted_suggestions.sort_by_key(|s| s.priority);
            
            for (i, suggestion) in sorted_suggestions.iter().enumerate() {
                output.push_str(&format!("{}. {} (Priority: {})\n", 
                    i + 1, suggestion.title, suggestion.priority));
                output.push_str(&format!("   {}\n", suggestion.description));
                if suggestion.quick_fix {
                    output.push_str("   This is a quick fix.\n");
                }
                output.push_str("\n");
            }
        }
        
        output
    }
}
