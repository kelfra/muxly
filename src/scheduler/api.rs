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
use tracing::{error, info, debug};
use uuid::Uuid;
use futures::future::BoxFuture;
use serde_json::json;
use serde_json::Value;

/// Configuration for the API scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSchedulerConfig {
    /// Whether the scheduler is enabled
    pub enabled: bool,
    /// Maximum number of concurrent jobs
    pub max_concurrent_jobs: Option<usize>,
    /// Job timeout in seconds
    pub job_timeout_seconds: Option<u64>,
    /// Maximum size of job execution history
    pub max_history_size: Option<usize>,
}

/// Job handler type
pub type JobHandler = Arc<dyn Fn(serde_json::Value) -> Result<serde_json::Value> + Send + Sync>;

/// A job registered in the API scheduler
#[derive(Clone)]
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
    /// Created at
    created_at: DateTime<Utc>,
    /// Updated at
    updated_at: DateTime<Utc>,
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
            config,
            jobs: RwLock::new(HashMap::new()),
            executions: RwLock::new(HashMap::new()),
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
            created_at: Utc::now(),
            updated_at: Utc::now(),
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
                job.updated_at = Utc::now();
                Ok(())
            })
            .unwrap_or_else(|| Err(anyhow!("Job with ID '{}' not found", id)))
    }
    
    /// Run a job with the given parameters
    pub async fn run_job(&self, job_id: String, parameters: Option<Value>) -> Result<()> {
        debug!("Running job {}", job_id);
        
        // Create the execution record
        let execution_id = Uuid::new_v4().to_string();
        let execution_start = Utc::now();
        
        let execution = JobExecution {
            id: execution_id.clone(),
            job_id: job_id.clone(),
            status: "Pending".to_string(),
            parameters: parameters.clone(),
            start_time: execution_start,
            end_time: None,
            result: None,
            error: None,
        };
        
        // Store the execution
        self.create_execution(job_id.clone(), execution).await?;
        
        // Clone self into an Arc to be moved into the task
        let self_arc = Arc::new(self.clone());
        let job_id_clone = job_id.clone();
        let execution_id_clone = execution_id.clone();
        let parameters_clone = parameters.clone();
        
        // Spawn a task to run the job
        tokio::spawn(async move {
            // Update execution status to Running
            {
                let mut execs = self_arc.executions.write().await;
                if let Some(exec) = execs.get_mut(&execution_id_clone) {
                    exec.status = "Running".to_string();
                }
            }
            
            // Get the job handler and run it
            let result = {
                let jobs = self_arc.jobs.read().await;
                match jobs.get(&job_id_clone) {
                    Some(job) => {
                        // Get the handler
                        let handler = job.handler.clone();
                        
                        // Run the handler with the parameters
                        handler(parameters_clone)
                    }
                    None => {
                        // Job not found
                        Err(anyhow!("Job not found"))
                    }
                }
            };
            
            // Update the execution with the result
            let execution_end = Utc::now();
            {
                let mut execs = self_arc.executions.write().await;
                if let Some(exec) = execs.get_mut(&execution_id_clone) {
                    exec.end_time = Some(execution_end);
                    match result {
                        Ok(res) => {
                            exec.status = "Completed".to_string();
                            exec.result = Some(res);
                        }
                        Err(e) => {
                            exec.status = "Failed".to_string();
                            exec.error = Some(e.to_string());
                        }
                    }
                }
            }
            
            // Update the job's last execution
            {
                let mut jobs = self_arc.jobs.write().await;
                if let Some(job) = jobs.get_mut(&job_id_clone) {
                    job.last_execution = Some(execution_id_clone);
                }
            }
        });
        
        Ok(())
    }
    
    /// Get job execution
    pub async fn get_execution(&self, id: &str) -> Result<JobExecution> {
        let executions = self.executions.read().await;
        
        executions
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow!("Execution with ID '{}' not found", id))
    }
    
    /// List history for a job
    pub async fn list_execution_history(&self, job_id: &str) -> Result<Vec<String>> {
        let executions_map = self.executions.read().await;
        let execution_ids: Vec<String> = executions_map
            .values()
            .filter(|e| e.job_id == job_id)
            .map(|e| e.id.clone())
            .collect();
        
        Ok(execution_ids)
    }

    /// Create a new execution record
    pub async fn create_execution(&self, job_id: String, execution: JobExecution) -> Result<()> {
        let mut executions = self.executions.write().await;
        executions.insert(execution.id.clone(), execution);
        
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&job_id) {
            job.last_execution = Some(execution.clone());
        }
        
        Ok(())
    }

    /// Cancel and clean up a job and all its executions
    async fn cancel_job_internal(&self, job_id: &str) -> Result<()> {
        // Remove the job
        let job = {
            let mut jobs = self.jobs.write().await;
            jobs.remove(job_id).ok_or_else(|| anyhow!("Job not found"))?
        };
        
        // Clean up executions for this job
        {
            let mut executions = self.executions.write().await;
            executions.retain(|_, execution| execution.job_id != job_id);
        }
        
        debug!("Job {} cancelled", job_id);
        Ok(())
    }
    
    /// Create routes for the API scheduler
    pub fn routes(self: Arc<Self>) -> Router {
        Router::new()
            .route("/jobs", get(list_jobs))
            .route("/jobs/:id", get(get_job))
            .route("/executions", get(list_executions))
            .route("/executions/:id", get(get_execution))
            .with_state(self)
    }
}

/// Standalone handler functions for axum compatibility
async fn list_jobs(
    State(scheduler): State<Arc<ApiScheduler>>,
) -> impl IntoResponse {
    let jobs = scheduler.get_jobs(None).await;
    Json(jobs).into_response()
}

async fn get_job(
    State(scheduler): State<Arc<ApiScheduler>>, 
    Path(job_id): Path<String>
) -> impl IntoResponse {
    let jobs = scheduler.jobs.read().await;
    match jobs.get(&job_id) {
        Some(job) => Json(job).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Job {} not found", job_id)}))
        ).into_response()
    }
}

async fn list_executions(
    State(scheduler): State<Arc<ApiScheduler>>,
) -> impl IntoResponse {
    let executions = scheduler.list_execution_history("").await;
    Json(executions).into_response()
}

async fn get_execution(
    State(scheduler): State<Arc<ApiScheduler>>,
    Path(execution_id): Path<String>
) -> impl IntoResponse {
    let executions = scheduler.executions.read().await;
    match executions.get(&execution_id) {
        Some(execution) => Json(execution).into_response(),
        None => (
            StatusCode::NOT_FOUND, 
            Json(json!({"error": format!("Execution {} not found", execution_id)}))
        ).into_response()
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
    match scheduler.run_job(id, request.parameters).await {
        Ok(_) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({
                "status": "success",
                "message": format!("Job {} started", id)
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

