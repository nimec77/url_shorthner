use std::sync::Arc;

use axum::{
    extract::{MatchedPath, Path, Request, State}, http::StatusCode, response::{IntoResponse, Response}, routing::{get, post}, Json, Router
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    app::{
        command::create_short_url::CreateShortUrlRepository,
        query::get_full_url::GetFullUrlRepository,
    }, di::Container, error::AppError, id_provider::IdProvider
};

#[derive(Deserialize, Serialize)]
struct ErrorResponse {
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_owned()),
            AppError::UrlParseError => (StatusCode::BAD_REQUEST, "Invalid URL".to_owned()),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

pub struct Server<I, R, Q>
where
    I: IdProvider + Send + Sync + 'static,
    R: CreateShortUrlRepository + Send + Sync + 'static,
    Q: GetFullUrlRepository + Send + Sync + 'static,
{
    port: u16,
    container: Arc<Container<I, R, Q>>,
}

impl<I, R, Q> Server<I, R, Q>
where
    I: IdProvider + Send + Sync + 'static,
    R: CreateShortUrlRepository + Send + Sync + 'static,
    Q: GetFullUrlRepository + Send + Sync + 'static,
{
    pub fn new(port: u16, container: Arc<Container<I, R, Q>>) -> Self {
        Self { port, container }
    }

    pub async fn run(self) {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "url_shortener=debug,tower_http=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();

        let router = get_router(self.container);
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

        axum::serve(listener, router).await.unwrap();
    }
}

fn get_router<I, R, Q>(container: Arc<Container<I, R, Q>>) -> Router
where
    I: IdProvider + Send + Sync + 'static,
    R: CreateShortUrlRepository + Send + Sync + 'static,
    Q: GetFullUrlRepository + Send + Sync + 'static,
{
    Router::new()
        .route("/{id}", get(get_full_url))
        .route("/", post(shorten_url))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &Request| {
                    let method = req.method();
                    let uri = req.uri();
                    let matched_path = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(|matched_path| matched_path.as_str());

                    tracing::debug_span!("request", %method, %uri, matched_path)
                })
                .on_failure(()),
        )
        .with_state(container)
}

#[derive(Deserialize, Serialize)]
struct CreateShortURLRequest {
    url: String,
}

#[derive(Deserialize, Serialize)]
struct ShortUrlResponse {
    id: String,
}

async fn shorten_url<I, R, Q>(
    State(container): State<Arc<Container<I, R, Q>>>,
    Json(input): Json<CreateShortURLRequest>,
) -> Result<Json<ShortUrlResponse>, AppError>
where
    I: IdProvider + Send + Sync + 'static,
    R: CreateShortUrlRepository + Send + Sync + 'static,
    Q: GetFullUrlRepository + Send + Sync + 'static,
{
    container
        .short_url_command
        .execute(&input.url)
        .await
        .map(|id| Json(ShortUrlResponse { id }))
}

#[derive(serde::Deserialize, serde::Serialize)]
struct FullUrlResponse {
    url: String,
}

impl From<String> for FullUrlResponse {
    fn from(url: String) -> Self {
        FullUrlResponse { url }
    }
}

async fn get_full_url<I, Q, R>(
    Path(id): Path<String>,
    State(container): State<Arc<Container<I, R, Q>>>,
) -> Result<Json<FullUrlResponse>, AppError>
where
    I: IdProvider + Send + Sync + 'static,
    R: CreateShortUrlRepository + Send + Sync + 'static,
    Q: GetFullUrlRepository + Send + Sync + 'static,
{
    container
        .get_full_url_query
        .execute(&id)
        .await
        .map(|url| Json(FullUrlResponse::from(url)))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        extract::Request,
        http::{Method, StatusCode, header},
    };
    use dashmap::DashMap;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::{adapters::in_memory::InMemoryRepository, id_provider::FakeIdProvider};

    use super::*;

    fn get_router_with_mock_container() -> Router {
        let store = Arc::new(DashMap::new());
        store.insert("test-id".to_owned(), "test-url".to_owned());
        store.insert("test-id-2".to_owned(), "test-url-2".to_owned());
        let repo = InMemoryRepository::new(store);

        let container = Container::new(
            FakeIdProvider::new("test-id".to_owned()),
            repo.clone(),
            repo,
        );

        get_router(Arc::new(container))
    }

    #[tokio::test]
    async fn test_get_full_url() {
        // Given
        let router = get_router_with_mock_container();

        // When
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/test-id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Then
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: FullUrlResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(body.url, "test-url");
    }

    #[tokio::test]
    async fn get_not_found() {
        // Given
        let router = get_router_with_mock_container();

        // When
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/not-found")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Then
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: ErrorResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(body.message, "Not found");
    }

    #[tokio::test]
    async fn test_get_different_id() {
        // Given
        let router = get_router_with_mock_container();

        // When
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/test-id-2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Then
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: FullUrlResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(body.url, "test-url-2");
    }

    #[tokio::test]
    async fn short_url() {
        // Given
        let router = get_router_with_mock_container();

        let create_short_url_request = CreateShortURLRequest {
            url: "https://example.com".to_owned(),
        };

        // When
        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/")
                    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        serde_json::to_string(&create_short_url_request).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Then
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: ShortUrlResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(body.id, "test-id");
    }

    #[tokio::test]
    async fn short_and_get() {
        // Given
        let store = Arc::new(DashMap::new());
        let repo = InMemoryRepository::new(store.clone());
        let repo2 = InMemoryRepository::new(store);

        let container = Arc::new(Container::new(
            FakeIdProvider::new("test-id".to_owned()),
            repo,
            repo2,
        ));

        let router1 = get_router(container.clone());
        let router2 = get_router(container.clone());

        let create_short_url_request = CreateShortURLRequest {
            url: "https://example.com/".to_owned(),
        };

        // When
        let resp1 = router1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/")
                    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        serde_json::to_string(&create_short_url_request).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let resp2 = router2
            .oneshot(
                Request::builder()
                    .uri("/test-id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Then
        assert_eq!(resp1.status(), StatusCode::OK);
        let body = resp1.into_body().collect().await.unwrap().to_bytes();
        let body: ShortUrlResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(body.id, "test-id");

        assert_eq!(resp2.status(), StatusCode::OK);
        let body = resp2.into_body().collect().await.unwrap().to_bytes();
        let body: FullUrlResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(body.url, "https://example.com/");
    }
}
