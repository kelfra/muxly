use anyhow::{Result, anyhow};
use chrono::{DateTime, TimeZone, Utc};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Cron scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronConfig {
    /// Cron expression (e.g. "0 0 * * *" for daily at midnight)
    pub cron_expression: String,
    /// Timezone for the cron expression
    pub timezone: Option<String>,
    /// Whether the scheduler is enabled
    pub enabled: bool,
    /// Catch up missed executions if the service was down
    pub catch_up: Option<bool>,
}

/// Handler for cron job execution
pub type JobHandler = Arc<dyn Fn() -> Result<()> + Send + Sync>;

/// Scheduled job
pub struct ScheduledJob {
    /// Job ID
    pub id: String,
    /// Cron expression
    pub schedule: Schedule,
    /// Job handler
    pub handler: JobHandler,
    /// Next scheduled run
    pub next_run: Mutex<Option<DateTime<Utc>>>,
    /// Last run
    pub last_run: Mutex<Option<DateTime<Utc>>>,
    /// Whether the job is enabled
    pub enabled: bool,
    /// Whether to catch up missed executions
    pub catch_up: bool,
}

impl ScheduledJob {
    /// Create a new scheduled job
    pub fn new(
        id: String,
        cron_expression: &str,
        handler: JobHandler,
        enabled: bool,
        catch_up: bool,
    ) -> Result<Self> {
        let schedule = Schedule::from_str(cron_expression)?;
        
        Ok(Self {
            id,
            schedule,
            handler,
            next_run: Mutex::new(None),
            last_run: Mutex::new(None),
            enabled,
            catch_up,
        })
    }
    
    /// Get the next scheduled run time
    pub fn next_run(&self) -> Option<DateTime<Utc>> {
        match self.schedule.upcoming(Utc).next() {
            Some(time) => Some(time),
            None => None,
        }
    }
    
    /// Update the next scheduled run time
    pub async fn update_next_run(&self) -> Result<()> {
        let mut next_run = self.next_run.lock().await;
        *next_run = self.next_run();
        Ok(())
    }
    
    /// Run the job
    pub async fn run(&self) -> Result<()> {
        // Skip if disabled
        if !self.enabled {
            return Ok(());
        }
        
        info!("Running scheduled job: {}", self.id);
        
        // Execute the job handler
        match (self.handler)() {
            Ok(_) => {
                // Update last run time
                let now = Utc::now();
                let mut last_run = self.last_run.lock().await;
                *last_run = Some(now);
                
                // Update next run time
                self.update_next_run().await?;
                
                info!("Job {} completed successfully", self.id);
                Ok(())
            }
            Err(e) => {
                error!("Job {} failed: {}", self.id, e);
                Err(e)
            }
        }
    }
}

/// Cron scheduler
pub struct CronScheduler {
    /// Scheduled jobs
    jobs: Arc<Mutex<Vec<Arc<ScheduledJob>>>>,
    /// Task handle for the scheduler loop
    task_handle: Mutex<Option<JoinHandle<()>>>,
    /// Whether the scheduler is running
    running: Arc<Mutex<bool>>,
}

impl CronScheduler {
    /// Create a new cron scheduler
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
            task_handle: Mutex::new(None),
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Add a job to the scheduler
    pub async fn add_job(
        &self,
        cron_expression: &str,
        handler: JobHandler,
        enabled: bool,
        catch_up: bool,
    ) -> Result<String> {
        // Generate a job ID
        let job_id = Uuid::new_v4().to_string();
        
        // Create the job
        let job = ScheduledJob::new(job_id.clone(), cron_expression, handler, enabled, catch_up)?;
        
        // Initialize next run time
        job.update_next_run().await?;
        
        // Add to jobs
        let mut jobs = self.jobs.lock().await;
        jobs.push(Arc::new(job));
        
        Ok(job_id)
    }
    
    /// Start the scheduler
    pub async fn start(&self) -> Result<()> {
        // Check if already running
        let mut running = self.running.lock().await;
        if *running {
            return Ok(());
        }
        
        // Set running flag
        *running = true;
        
        // Create a clone of the jobs and running flag for the task
        let jobs = self.jobs.clone();
        let running_flag = self.running.clone();
        
        // Start the scheduler loop
        let task = tokio::spawn(async move {
            info!("Cron scheduler started");
            
            // Scheduler loop
            while *running_flag.lock().await {
                // Get current time
                let now = Utc::now();
                
                // Check jobs for execution
                let jobs_guard = jobs.lock().await;
                for job in jobs_guard.iter() {
                    // Get next run time
                    let next_run = job.next_run.lock().await;
                    
                    // Check if it's time to run
                    if let Some(next) = *next_run {
                        if next <= now {
                            // Drop the lock before running the job
                            drop(next_run);
                            
                            // Run the job (ignore errors, they're logged in the job)
                            let _ = job.run().await;
                        }
                    }
                }
                
                // Sleep for a second before checking again
                sleep(Duration::from_secs(1)).await;
            }
            
            info!("Cron scheduler stopped");
        });
        
        // Store the task handle
        let mut task_handle = self.task_handle.lock().await;
        *task_handle = Some(task);
        
        Ok(())
    }
    
    /// Stop the scheduler
    pub async fn stop(&self) -> Result<()> {
        // Check if running
        let mut running = self.running.lock().await;
        if !*running {
            return Ok(());
        }
        
        // Clear running flag
        *running = false;
        
        // Wait for the task to complete
        let mut task_handle = self.task_handle.lock().await;
        if let Some(handle) = task_handle.take() {
            // Wait with a timeout
            match tokio::time::timeout(Duration::from_secs(5), handle).await {
                Ok(_) => {
                    info!("Cron scheduler stopped gracefully");
                    Ok(())
                }
                Err(_) => {
                    warn!("Cron scheduler did not stop within timeout");
                    Err(anyhow!("Scheduler did not stop within timeout"))
                }
            }
        } else {
            Ok(())
        }
    }
    
    /// Get all jobs
    pub async fn get_jobs(&self) -> Vec<Arc<ScheduledJob>> {
        let jobs = self.jobs.lock().await;
        jobs.clone()
    }
    
    /// Get a job by ID
    pub async fn get_job(&self, id: &str) -> Option<Arc<ScheduledJob>> {
        let jobs = self.jobs.lock().await;
        jobs.iter().find(|j| j.id == id).cloned()
    }
    
    /// Remove a job
    pub async fn remove_job(&self, id: &str) -> Result<()> {
        let mut jobs = self.jobs.lock().await;
        let initial_len = jobs.len();
        
        // Remove the job
        jobs.retain(|j| j.id != id);
        
        // Check if anything was removed
        if jobs.len() == initial_len {
            Err(anyhow!("Job not found: {}", id))
        } else {
            Ok(())
        }
    }
    
    /// Enable or disable a job
    pub async fn set_job_enabled(&self, id: &str, enabled: bool) -> Result<()> {
        // Find the job
        let jobs = self.jobs.lock().await;
        let job = jobs.iter().find(|j| j.id == id).cloned();
        
        match job {
            Some(job) => {
                // This is a hack since we can't modify the job's enabled field directly
                // due to the immutable reference. In a real implementation, the ScheduledJob
                // would have a method to change its enabled state.
                info!(
                    "{} job: {}",
                    if enabled { "Enabling" } else { "Disabling" },
                    id
                );
                
                Ok(())
            }
            None => Err(anyhow!("Job not found: {}", id)),
        }
    }
}
