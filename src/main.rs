use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;

mod api;
mod config;
mod services;
mod types;

use config::Config;
use services::p2p::{P2PService, TwapMessage};
use services::redis_client::RedisClient;
use services::{Indexer, SigningService};
use tokio::sync::mpsc;

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

    // Create channel for P2P message broadcasting
    let (p2p_sender, p2p_receiver) = mpsc::unbounded_channel();

    // Initialize P2P service
    let p2p_service = P2PService::new(
        config.p2p.listen_address, // Remove .clone()
        config.p2p.bootstrap_peers,
    )
    .await?;

    println!(
        "P2P Service initialized with peer ID: {:?}",
        p2p_service.peer_id()
    );

    // Start P2P service without Arc
    let p2p_handle = tokio::spawn(async move {
        if let Err(e) = p2p_service.run(p2p_receiver).await {
            eprintln!("P2P service error: {}", e);
        }
    });

    // Start the API server
    let app = api::create_router(api_redis_client, signing_service, p2p_sender);

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

    // Wait for all tasks
    let _ = tokio::try_join!(indexer_handle, server_handle, p2p_handle)?;

    Ok(())
}
