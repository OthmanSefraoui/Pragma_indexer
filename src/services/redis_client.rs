use crate::types::spot_entry::SpotEntry;
use anyhow::Result;
use redis::AsyncCommands;

#[derive(Clone)]
pub struct RedisClient {
    client: redis::Client,
}

impl RedisClient {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(RedisClient { client })
    }

    pub async fn check_connection(&self) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        // Try a simple PING command
        redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| anyhow::anyhow!("Redis connection failed: {}", e))?;
        Ok(())
    }

    pub async fn store_spot_entry(&self, entry: &SpotEntry) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = entry.redis_key();
        let json = serde_json::to_string(entry)?;

        // Store in a sorted set with timestamp as score for easy retrieval
        conn.zadd(key, &json, entry.timestamp.parse::<f64>()?)
            .await?;
        Ok(())
    }

    pub async fn get_spot_entries(
        &self,
        pair_id: &str,
        start_time: Option<f64>,
        end_time: Option<f64>,
    ) -> Result<Vec<SpotEntry>> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("spot:{}", pair_id);

        let entries: Vec<String> = match (start_time, end_time) {
            (Some(start), Some(end)) => {
                // Get entries within time range
                conn.zrangebyscore(key, start, end).await?
            }
            _ => {
                // Get all entries
                conn.zrange(key, 0, -1).await?
            }
        };

        entries
            .into_iter()
            .map(|json| Ok(serde_json::from_str(&json)?))
            .collect()
    }

    pub async fn compute_twap(&self, pair_id: &str, period: u64) -> Result<Option<f64>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as f64;
        let start_time = now - period as f64;

        let entries = self
            .get_spot_entries(pair_id, Some(start_time), Some(now))
            .await?;

        if entries.is_empty() {
            return Ok(None);
        }

        let sum: f64 = entries
            .iter()
            .map(|entry| entry.price.parse::<f64>().unwrap_or(0.0))
            .sum();

        Ok(Some(sum / entries.len() as f64))
    }
}
