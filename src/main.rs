use anyhow::Result;
use std::net::SocketAddr;

mod api;
mod config;
mod services;
mod types;

use config::Config;
use services::redis_client::RedisClient;
use services::{Indexer, SigningService};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting application...");
    let config = match Config::new() {
        Ok(config) => {
            println!("Configuration loaded successfully");
            config
        }
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            return Err(e);
        }
    };
    println!("Configuration loaded successfully");
    let redis_client = RedisClient::new(&config.redis_url)?;

    let api_redis_client = redis_client.clone();
    let signing_service = SigningService::new(&config.private_key)?;
    let indexer_config = config.clone();
    // Start the indexer in a separate task
    let indexer_handle = tokio::spawn(async move {
        println!("Starting indexer service...");

        let indexer = Indexer::new(indexer_config.clone(), redis_client);

        println!("Indexer created, starting to run...");

        if let Err(e) = indexer.run().await {
            eprintln!("Indexer error: {}", e);
        }
    });

    // Start the API server
    let app = api::create_router(api_redis_client, signing_service);

    let addr = SocketAddr::new(config.server_host.parse()?, config.server_port);
    println!("API server starting on {}", addr);

    let server = axum::Server::bind(&addr).serve(app.into_make_service());

    println!(
        "Server running at http://{}:{}",
        config.server_host, config.server_port
    );
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.await {
            eprintln!("Server error: {}", e);
        }
    });

    // Wait for both tasks
    let _ = tokio::try_join!(indexer_handle, server_handle)?;

    Ok(())
}
