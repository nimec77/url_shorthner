use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;

use crate::{
    app::{
        command::create_short_url::CreateShortUrlRepository,
        query::get_full_url::GetFullUrlRepository,
    },
    error::AppError,
};

#[derive(Debug, Clone)]
pub struct InMemoryRepository {
    store: Arc<DashMap<String, String>>,
}

impl InMemoryRepository {
    pub fn new(store: Arc<DashMap<String, String>>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl CreateShortUrlRepository for InMemoryRepository {
    async fn save(&self, full_url: String, id: String) -> Result<(), AppError> {
        self.store.insert(id, full_url);

        Ok(())
    }
}

impl GetFullUrlRepository for InMemoryRepository {
    async fn get(&self, id: &str) -> Result<String, AppError> {
        self.store
            .get(id)
            .map(|url| url.clone())
            .ok_or(AppError::NotFound)
    }
}
