use async_trait::async_trait;
use url::Url;

use crate::{error::AppError, id_provider::IdProvider};

#[mockall::automock]
#[async_trait]
pub trait CreateShortUrlRepository {
    async fn save<'a>(&'a self, full_url: String, id: String) -> Result<(), AppError>;
}

pub struct CreateShortUrlCommand<I, R>
where
    I: IdProvider,
    R: CreateShortUrlRepository,
{
    id_provider: I,
    repository: R,
}

impl<I, R> CreateShortUrlCommand<I, R>
where
    I: IdProvider,
    R: CreateShortUrlRepository,
{
    pub fn new(id_provider: I, repository: R) -> Self {
        Self {
            id_provider,
            repository,
        }
    }

    pub async fn execute(&self, full_url: &str) -> Result<String, AppError> {
        let parsed_url = Url::parse(full_url).map_err(|_| AppError::UrlParseError)?;
        let id = self.id_provider.provide();
        self.repository
            .save(parsed_url.to_string(), id.clone())
            .await?;

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use dashmap::DashMap;

    use crate::{
        adapters::in_memory::InMemoryRepository,
        id_provider::{FakeIdProvider, MockIdProvider, NanoIdProvider},
    };

    use super::*;

    #[tokio::test]
    async fn get_short_url() {
        // Given
        let id_provider = FakeIdProvider::new("123".to_owned());
        let store = Arc::new(DashMap::new());
        let repository = InMemoryRepository::new(store);
        let create_short_url = CreateShortUrlCommand::new(id_provider, repository);

        // When
        let result = create_short_url.execute("https://www.google.com").await;

        // Then
        assert_ne!(result, Ok("".to_owned()));
    }

    #[tokio::test]
    async fn get_two_different_short_urls() {
        // Given
        let id_provider = NanoIdProvider;
        let store = Arc::new(DashMap::new());
        let repository = InMemoryRepository::new(store);
        let create_short_url = CreateShortUrlCommand::new(id_provider, repository);

        // When
        let result1 = create_short_url
            .execute("https://www.google.com")
            .await
            .unwrap();

        let result2 = create_short_url
            .execute("https://www.example.com")
            .await
            .unwrap();

        // Then
        assert_ne!(result1, result2);
    }

    #[tokio::test]
    async fn after_save_store_should_have_one_item() {
        // Given
        let id_provider = NanoIdProvider;
        let store: Arc<DashMap<String, String>> = Arc::new(DashMap::new());
        let repository = InMemoryRepository::new(store.clone());
        let create_short_url = CreateShortUrlCommand::new(id_provider, repository);

        // When
        let id = create_short_url
            .execute("https://www.google.com")
            .await
            .unwrap();

        // Then
        assert_eq!(store.len(), 1);
        let full_url = store.get(&id).unwrap();
        assert_eq!(full_url.value(), "https://www.google.com/");
    }

    #[tokio::test]
    async fn get_short_url_with_mock() {
        // Given
        let mut stub_id_provider = MockIdProvider::new();
        stub_id_provider
            .expect_provide()
            .returning(|| "123".to_owned())
            .times(1);

        let mut mock_repo = MockCreateShortUrlRepository::new();
        mock_repo.expect_save().returning(|_, _| Ok(())).times(1);
        let sut = CreateShortUrlCommand::new(stub_id_provider, mock_repo);

        // When
        let result = sut.execute("https://www.google.com").await;

        // Then
        assert_eq!(result, Ok("123".to_owned()));
    }
}
