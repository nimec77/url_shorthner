pub mod command;
pub mod query;

#[cfg(test)]
mod tests {
    use crate::{
        adapters::in_memory::InMemoryRepository,
        app::{
            command::create_short_url::CreateShortUrlCommand, query::get_full_url::GetFullUrlQuery,
        },
        id_provider::FakeIdProvider,
    };
    use dashmap::DashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn create_and_get_short_url() {
        // Given
        let store = Arc::new(DashMap::new());
        let repository = InMemoryRepository::new(store);
        let id_provider = FakeIdProvider::new("123".to_owned());
        let create_command = CreateShortUrlCommand::new(id_provider, repository.clone());
        let get_query = GetFullUrlQuery::new(repository);

        // When
        let short_url = create_command
            .execute("https://www.google.com".to_owned())
            .await;
        let full_url = get_query.execute(&short_url.unwrap()).await.unwrap();

        // Then
        assert_eq!(full_url, "https://www.google.com");
    }
}
