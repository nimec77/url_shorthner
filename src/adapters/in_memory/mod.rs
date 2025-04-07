use std::{pin::Pin, sync::Arc};

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

impl CreateShortUrlRepository for InMemoryRepository {
    fn save<'a>(&'a self, full_url: String, id: String) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            self.store.insert(id, full_url);
            Ok(())
        })
    }
}

impl GetFullUrlRepository for InMemoryRepository {
    fn get<'a>(&'a self, id: &'a str) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'a>> {
        Box::pin(async move {
            self.store
                .get(id)
                .map(|url| url.clone())
                .ok_or(AppError::NotFound)
        })
    }
}
