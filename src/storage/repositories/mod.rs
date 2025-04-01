use anyhow::Result;
use std::sync::Arc;
use crate::storage::database::DatabasePool;

/// Base repository trait for CRUD operations
pub trait Repository<T> {
    async fn find_all(&self) -> Result<Vec<T>>;
    async fn find_by_id(&self, id: &str) -> Result<Option<T>>;
    async fn create(&self, item: T) -> Result<T>;
    async fn update(&self, id: &str, item: T) -> Result<T>;
    async fn delete(&self, id: &str) -> Result<bool>;
}

/// Factory for creating repositories
pub struct RepositoryFactory {
    pool: Arc<DatabasePool>,
}

impl RepositoryFactory {
    pub fn new(pool: Arc<DatabasePool>) -> Self {
        Self { pool }
    }
} 