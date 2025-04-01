mod database;
mod email;
mod file;
mod prometheus;
mod slack;
mod storage;
mod webhook;

pub use database::DatabaseDestination;
pub use email::EmailDestination;
pub use file::FileDestination;
pub use prometheus::PrometheusDestination;
pub use slack::SlackDestination;
pub use storage::S3Destination;
pub use webhook::WebhookDestination; 