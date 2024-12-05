use anyhow::Result;
mod services;
use services::Indexer;
mod api;
mod types;
use services::redis_client::RedisClient;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    let apibara_key = std::env::var("APIBARA_API_KEY").expect("APIBARA_API_KEY must be set");

    let contract_address =
        "0x36031daa264c24520b11d93af622c848b2499b66b41d611bac95e13cfca131a".to_string();
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = RedisClient::new(&redis_url)?;

    let api_redis_client = redis_client.clone();
    // Start the indexer in a separate task
    let indexer_handle = tokio::spawn(async move {
        let api_key = std::env::var("APIBARA_API_KEY").expect("APIBARA_API_KEY must be set");

        let contract_address =
            "0x36031daa264c24520b11d93af622c848b2499b66b41d611bac95e13cfca131a".to_string();

        let indexer = Indexer::new(api_key, 366750, contract_address, redis_url);

        if let Err(e) = indexer.run().await {
            eprintln!("Indexer error: {}", e);
        }
    });

    // Start the API server
    let app = api::create_router(api_redis_client);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("API server listening on {}", addr);

    let server_handle = tokio::spawn(async move {
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    // Wait for both tasks
    let _ = tokio::try_join!(indexer_handle, server_handle)?;

    Ok(())
}
