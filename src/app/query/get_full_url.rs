use crate::error::AppError;

pub trait GetFullUrlRepository {
    fn get<'a>(
        &'a self,
        id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, AppError>> + Send + 'a>>;
}

pub struct GetFullUrlQuery<R>
where
    R: GetFullUrlRepository,
{
    repository: R,
}

impl<R> GetFullUrlQuery<R>
where
    R: GetFullUrlRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, id: &str) -> Result<String, AppError> {
        self.repository.get(id).await
    }
}

#[cfg(test)]
mod tests {
    use std::{pin::Pin, sync::Arc};

    use dashmap::DashMap;

    use crate::adapters::in_memory::InMemoryRepository;

    use super::*;

    #[tokio::test]
    async fn get_full_url() {
        // Given
        struct FakeRepository;

        impl GetFullUrlRepository for FakeRepository {
            fn get<'a>(
                &'a self,
                _id: &'a str,
            ) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'a>> {
                Box::pin(async move { Ok("https://www.google.com".to_owned()) })
            }
        }
        let repository = FakeRepository;
        let get_full_url = GetFullUrlQuery::new(repository);

        // When
        let result = get_full_url.execute("123").await;

        // Then
        assert_eq!(result, Ok("https://www.google.com".to_owned()));
    }

    #[tokio::test]
    async fn get_from_in_memory_repository() {
        // Given
        let store: Arc<DashMap<String, String>> = Arc::new(DashMap::new());
        store.insert("123".to_owned(), "https://www.google.com".to_owned());
        let repository = InMemoryRepository::new(store);
        let get_full_url = GetFullUrlQuery::new(repository);

        // When
        let result = get_full_url.execute("123").await;

        // Then
        assert_eq!(result, Ok("https://www.google.com".to_owned()));
    }

    #[tokio::test]
    async fn get_two_different_full_urls() {
        // Given
        let store: Arc<DashMap<String, String>> = Arc::new(DashMap::new());
        store.insert("123".to_owned(), "https://www.google.com".to_owned());
        store.insert("456".to_owned(), "https://www.example.com".to_owned());
        let repository = InMemoryRepository::new(store);
        let get_full_url = GetFullUrlQuery::new(repository);

        // When
        let result1 = get_full_url.execute("123").await;
        let result2 = get_full_url.execute("456").await;

        // Then
        assert_eq!(result1, Ok("https://www.google.com".to_owned()));
        assert_eq!(result2, Ok("https://www.example.com".to_owned()));
    }
}
