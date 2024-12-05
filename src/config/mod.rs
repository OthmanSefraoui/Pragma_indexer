// src/config/mod.rs

use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub redis_url: String,
    pub apibara_api_key: String,
    pub contract_address: String,
    pub server_host: String,
    pub server_port: u16,
    pub starting_block: u64,
}

impl Config {
    pub fn new() -> Result<Self> {
        dotenv::dotenv().ok();

        Ok(Config {
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),

            apibara_api_key: env::var("APIBARA_API_KEY").context("APIBARA_API_KEY must be set")?,

            contract_address: env::var("CONTRACT_ADDRESS").unwrap_or_else(|_| {
                println!("Using default contract address");
                "0x36031daa264c24520b11d93af622c848b2499b66b41d611bac95e13cfca131a".to_string()
            }),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),

            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("SERVER_PORT must be a valid number")?,

            starting_block: env::var("STARTING_BLOCK")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .context("STARTING_BLOCK must be a valid number")?,
        })
    }
}

// Constants for event selectors
pub const SUBMITTED_SPOT_ENTRY_SELECTOR: &str =
    "0x280bb2099800026f90c334a3a23888ffe718a2920ffbbf4f44c6d3d5efb613c";

// Redis key prefixes
pub const REDIS_KEY_PREFIX_SPOT: &str = "spot:";
