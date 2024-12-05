use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::services::redis_client::RedisClient;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: String,
    redis_connection: bool,
}

#[derive(Debug, Deserialize)]
pub struct TwapQuery {
    pair_id: String,
    period: Option<u64>, // period in seconds, optional with default
}

#[derive(Debug, Serialize)]
pub struct TwapResponse {
    pair_id: String,
    twap: f64,
    period: u64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    error: String,
}

pub struct ApiState {
    pub redis_client: RedisClient,
}

pub fn create_router(redis_client: RedisClient) -> Router {
    let state = Arc::new(ApiState { redis_client });

    Router::new()
        .route("/health", get(health_check))
        .route("/api/get_data", get(get_twap))
        .with_state(state)
}

async fn health_check(State(state): State<Arc<ApiState>>) -> Json<HealthResponse> {
    // Check Redis connection
    let redis_status = state.redis_client.check_connection().await.is_ok();

    Json(HealthResponse {
        status: "up".to_string(),
        redis_connection: redis_status,
    })
}

async fn get_twap(
    State(state): State<Arc<ApiState>>,
    Query(params): Query<TwapQuery>,
) -> Result<Json<TwapResponse>, (StatusCode, Json<ErrorResponse>)> {
    let period = params.period.unwrap_or(3600); // Default to 1 hour
    println!("{}", params.pair_id);
    let twap = state
        .redis_client
        .compute_twap(&params.pair_id, period)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to compute TWAP: {}", e),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("No data found for pair {}", params.pair_id),
                }),
            )
        })?;

    Ok(Json(TwapResponse {
        pair_id: params.pair_id,
        twap,
        period,
    }))
}