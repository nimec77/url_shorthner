use crate::id_provider::IdProvider;

pub trait CreateShortUrlRepository {
    fn save(&self, full_url: String, id: String) -> Result<(), String>;
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

    pub async fn execute(&self, full_url: String) -> Result<String, String> {
        let id = self.id_provider.provide();
        self.repository.save(full_url, id.clone())?;

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use dashmap::DashMap;

    use crate::{
        adapters::in_memory::InMemoryRepository,
        id_provider::{FakeIdProvider, NanoIdProvider},
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
        let result = create_short_url
            .execute("https://www.google.com".to_owned())
            .await;

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
            .execute("https://www.google.com".to_owned())
            .await
            .unwrap();

        let result2 = create_short_url
            .execute("https://www.example.com".to_owned())
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
            .execute("https://www.google.com".to_owned())
            .await
            .unwrap();

        // Then
        assert_eq!(store.len(), 1);
        let full_url = store.get(&id).unwrap();
        assert_eq!(full_url.value(), "https://www.google.com");
    }
}
