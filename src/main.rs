use std::sync::Arc;

use adapters::in_memory::InMemoryRepository;
use dashmap::DashMap;
use di::Container;
use id_provider::NanoIdProvider;
use ports::http_api::Server;

pub mod adapters;
pub mod app;
pub mod di;
pub mod id_provider;
pub mod ports;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let store = Arc::new(DashMap::new());
    let id_provider = NanoIdProvider;
    let in_memory_repository = InMemoryRepository::new(store);
    let container = Arc::new(Container::new(
        id_provider,
        in_memory_repository.clone(),
        in_memory_repository,
    ));

    let server = Server::new(3000, container);

    server.run().await;
}
