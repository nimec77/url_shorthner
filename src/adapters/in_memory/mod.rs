use std::sync::Arc;

use dashmap::DashMap;

use crate::app::{
    command::create_short_url::CreateShortUrlRepository, query::get_full_url::GetFullUrlRepository,
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

impl CreateShortUrlRepository for InMemoryRepository {
    fn save(&self, full_url: String, id: String) -> Result<(), String> {
        self.store.insert(id, full_url);

        Ok(())
    }
}

impl GetFullUrlRepository for InMemoryRepository {
    fn get(&self, id: &str) -> Result<String, String> {
        self.store
            .get(id)
            .map(|url| url.clone())
            .ok_or("Not found".to_owned())
    }
}
