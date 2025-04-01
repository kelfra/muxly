// Placeholder for schedule storage models
#[derive(Debug)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub cron_expression: String,
    pub enabled: bool,
} 