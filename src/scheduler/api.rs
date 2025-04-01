use anyhow::{Result, anyhow};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use uuid::Uuid;

/// API scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSchedulerConfig {
    /// Whether the API scheduler is enabled
    pub enabled: bool,
}

/// Job handler type
pub type JobHandler = Arc<dyn Fn(serde_json::Value) -> Result<serde_json::Value> + Send + Sync>;

/// Registered job
struct RegisteredJob {
    /// Job ID
    id: String,
    /// Job name
    name: String,
    /// Job description
    description: Option<String>,
    /// Job handler
    handler: JobHandler,
    /// Whether the job is enabled
    enabled: bool,
    /// Last execution
    last_execution: Option<JobExecution>,
}

/// Job execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecution {
    /// Execution ID
    pub id: String,
    /// Job ID
    pub job_id: String,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time
    pub end_time: Option<DateTime<Utc>>,
    /// Status
    pub status: JobStatus,
    /// Parameters
    pub parameters: Option<serde_json::Value>,
    /// Result
    pub result: Option<serde_json::Value>,
    /// Error message
    pub error: Option<String>,
}

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job is pending execution
    Pending,
    /// Job is running
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed
    Failed,
}

/// Job description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDescription {
    /// Job ID
    pub id: String,
    /// Job name
    pub name: String,
    /// Job description
    pub description: Option<String>,
    /// Whether the job is enabled
    pub enabled: bool,
    /// Last execution
    pub last_execution: Option<JobExecution>,
}

/// Create job request
#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    /// Job name
    pub name: String,
    /// Job description
    pub description: Option<String>,
    /// Whether the job is enabled
    pub enabled: Option<bool>,
}

/// Run job request
#[derive(Debug, Deserialize)]
pub struct RunJobRequest {
    /// Parameters for the job
    pub parameters: Option<serde_json::Value>,
}

/// Job list query
#[derive(Debug, Deserialize)]
pub struct JobListQuery {
    /// Filter by enabled status
    pub enabled: Option<bool>,
}

/// API scheduler
pub struct ApiScheduler {
    /// Registered jobs
    jobs: RwLock<HashMap<String, RegisteredJob>>,
    /// Job executions
    executions: RwLock<HashMap<String, JobExecution>>,
    /// Configuration
    config: ApiSchedulerConfig,
}

impl ApiScheduler {
    /// Create a new API scheduler
    pub fn new(config: ApiSchedulerConfig) -> Self {
        Self {
            jobs: RwLock::new(HashMap::new()),
            executions: RwLock::new(HashMap::new()),
            config,
        }
    }
    
    /// Register a new job
    pub async fn register_job(
        &self,
        name: &str,
        description: Option<String>,
        handler: JobHandler,
        enabled: bool,
    ) -> Result<String> {
        // Skip if scheduler is disabled
        if !self.config.enabled {
            return Err(anyhow!("API scheduler is disabled"));
        }
        
        // Generate a job ID
        let id = Uuid::new_v4().to_string();
        
        // Create a registered job
        let job = RegisteredJob {
            id: id.clone(),
            name: name.to_string(),
            description,
            handler,
            enabled,
            last_execution: None,
        };
        
        // Add to jobs
        let mut jobs = self.jobs.write().await;
        jobs.insert(id.clone(), job);
        
        Ok(id)
    }
    
    /// Unregister a job
    pub async fn unregister_job(&self, id: &str) -> Result<()> {
        // Skip if scheduler is disabled
        if !self.config.enabled {
            return Err(anyhow!("API scheduler is disabled"));
        }
        
        let mut jobs = self.jobs.write().await;
        
        if jobs.remove(id).is_none() {
            Err(anyhow!("Job with ID '{}' not found", id))
        } else {
            Ok(())
        }
    }
    
    /// Get all registered jobs
    pub async fn get_jobs(&self, enabled_filter: Option<bool>) -> Vec<JobDescription> {
        let jobs = self.jobs.read().await;
        
        jobs.values()
            .filter(|job| {
                if let Some(enabled) = enabled_filter {
                    job.enabled == enabled
                } else {
                    true
                }
            })
            .map(|job| JobDescription {
                id: job.id.clone(),
                name: job.name.clone(),
                description: job.description.clone(),
                enabled: job.enabled,
                last_execution: job.last_execution.clone(),
            })
            .collect()
    }
    
    /// Get a job by ID
    pub async fn get_job(&self, id: &str) -> Result<JobDescription> {
        let jobs = self.jobs.read().await;
        
        jobs.get(id)
            .map(|job| JobDescription {
                id: job.id.clone(),
                name: job.name.clone(),
                description: job.description.clone(),
                enabled: job.enabled,
                last_execution: job.last_execution.clone(),
            })
            .ok_or_else(|| anyhow!("Job with ID '{}' not found", id))
    }
    
    /// Enable or disable a job
    pub async fn set_job_enabled(&self, id: &str, enabled: bool) -> Result<()> {
        // Skip if scheduler is disabled
        if !self.config.enabled {
            return Err(anyhow!("API scheduler is disabled"));
        }
        
        let mut jobs = self.jobs.write().await;
        
        jobs.get_mut(id)
            .map(|job| {
                job.enabled = enabled;
                Ok(())
            })
            .unwrap_or_else(|| Err(anyhow!("Job with ID '{}' not found", id)))
    }
    
    /// Run a job
    pub async fn run_job(
        &self,
        id: &str,
        parameters: Option<serde_json::Value>,
    ) -> Result<String> {
        // Skip if scheduler is disabled
        if !self.config.enabled {
            return Err(anyhow!("API scheduler is disabled"));
        }
        
        // Find the job
        let handler = {
            let jobs = self.jobs.read().await;
            let job = jobs
                .get(id)
                .ok_or_else(|| anyhow!("Job with ID '{}' not found", id))?;
            
            if !job.enabled {
                return Err(anyhow!("Job is disabled"));
            }
            
            job.handler.clone()
        };
        
        // Create execution
        let execution_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let mut execution = JobExecution {
            id: execution_id.clone(),
            job_id: id.to_string(),
            start_time: now,
            end_time: None,
            status: JobStatus::Running,
            parameters: parameters.clone(),
            result: None,
            error: None,
        };
        
        // Store execution
        {
            let mut executions = self.executions.write().await;
            executions.insert(execution_id.clone(), execution.clone());
        }
        
        // Run the job in a new task
        let executions = self.executions.clone();
        let jobs = self.jobs.clone();
        
        tokio::spawn(async move {
            let result = handler(parameters.unwrap_or(serde_json::Value::Null));
            let now = Utc::now();
            
            // Update execution
            {
                let mut executions_guard = executions.write().await;
                if let Some(exec) = executions_guard.get_mut(&execution_id) {
                    exec.end_time = Some(now);
                    
                    match result {
                        Ok(result_value) => {
                            exec.status = JobStatus::Completed;
                            exec.result = Some(result_value);
                        }
                        Err(e) => {
                            exec.status = JobStatus::Failed;
                            exec.error = Some(format!("{}", e));
                        }
                    }
                }
            }
            
            // Update job's last execution
            {
                let mut jobs_guard = jobs.write().await;
                if let Some(job) = jobs_guard.get_mut(id) {
                    let executions_guard = executions.read().await;
                    job.last_execution = executions_guard.get(&execution_id).cloned();
                }
            }
        });
        
        Ok(execution_id)
    }
    
    /// Get job execution
    pub async fn get_execution(&self, id: &str) -> Result<JobExecution> {
        let executions = self.executions.read().await;
        
        executions
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow!("Execution with ID '{}' not found", id))
    }
    
    /// Returns all routes for the scheduler API
    pub fn routes(self: Arc<Self>) -> Router {
        let self_clone = self.clone();
        
        // Create routes for the scheduler API
        Router::new()
            .route("/jobs", get(Self::list_jobs))
            .route("/jobs", post(Self::create_job))
            .route("/jobs/:id", get(Self::get_job))
            .route("/jobs/:id/run", post(Self::run_job))
            .route("/jobs/:id/enable", post(Self::enable_job))
            .route("/jobs/:id/disable", post(Self::disable_job))
            .route("/executions/:id", get(Self::get_execution))
            .with_state(self_clone)
    }

    /// Lists all jobs in the scheduler
    async fn list_jobs(State(scheduler): State<Arc<Self>>) -> impl IntoResponse {
        let jobs = scheduler.jobs.read().await;
        
        let jobs: Vec<JobDescription> = jobs.values()
            .map(|job| job.into())
            .collect();
        
        (StatusCode::OK, Json(jobs))
    }

    /// Get job details
    async fn get_job(
        State(scheduler): State<Arc<Self>>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        match scheduler.get_job(&id).await {
            Ok(job) => (StatusCode::OK, Json(job)),
            Err(_) => {
                let error_response = serde_json::json!({
                    "error": format!("Job not found: {}", id)
                });
                (StatusCode::NOT_FOUND, Json(error_response))
            }
        }
    }

    /// Get execution details
    async fn get_execution(
        State(scheduler): State<Arc<Self>>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        match scheduler.get_execution(&id).await {
            Ok(execution) => (StatusCode::OK, Json(execution)),
            Err(_) => {
                let error_response = serde_json::json!({
                    "error": format!("Execution not found: {}", id)
                });
                (StatusCode::NOT_FOUND, Json(error_response))
            }
        }
    }

    /// Create a new async task for the job handler
    async fn spawn_job_task(&self, job_id: String) -> Result<String> {
        let job = {
            let jobs = self.jobs.read().await;
            jobs.get(&job_id).cloned().ok_or_else(|| anyhow!("Job not found"))?
        };
        
        let executions = self.executions.clone();
        let execution_id = Uuid::new_v4().to_string();
        
        // Create task
        tokio::spawn(async move {
            // Implementation details
        });
        
        Ok(execution_id)
    }
}

/// Handler for enabling a job
async fn enable_job(
    State(scheduler): State<Arc<ApiScheduler>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match scheduler.set_job_enabled(&id, true).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "message": format!("Job {} enabled", id)
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("{}", e)
            })),
        ),
    }
}

/// Handler for disabling a job
async fn disable_job(
    State(scheduler): State<Arc<ApiScheduler>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match scheduler.set_job_enabled(&id, false).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "message": format!("Job {} disabled", id)
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("{}", e)
            })),
        ),
    }
}

/// Handler for running a job
async fn run_job(
    State(scheduler): State<Arc<ApiScheduler>>,
    Path(id): Path<String>,
    Json(request): Json<RunJobRequest>,
) -> impl IntoResponse {
    match scheduler.run_job(&id, request.parameters).await {
        Ok(execution_id) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({
                "status": "success",
                "message": format!("Job {} started", id),
                "execution_id": execution_id
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("{}", e)
            })),
        ),
    }
}
