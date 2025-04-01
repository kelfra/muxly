// Placeholder for sync history storage models
#[derive(Debug)]
pub struct SyncHistory {
    pub id: String,
    pub connector_id: String,
    pub status: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
} 