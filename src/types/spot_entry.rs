use apibara_core::starknet::v1alpha2::{Event, FieldElement};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use starknet::core::types::Felt;

#[derive(Debug, Serialize, Deserialize)]
pub struct SpotEntry {
    pub timestamp: String,
    pub source: String,
    pub publisher: String,
    pub price: String,
    pub pair_id: String,
    pub volume: String,
    pub block_number: u64,
}

impl SpotEntry {
    pub fn from_event(event: &Event, block_number: u64) -> Option<Self> {
        if event.data.len() < 6 {
            return None;
        }

        let timestamp = Felt::from_bytes_be(&event.data[0].to_bytes()).to_bigint();
        let source = int_to_ascii(&Felt::from_bytes_be(&event.data[1].to_bytes()).to_bigint());
        let publisher = int_to_ascii(&Felt::from_bytes_be(&event.data[2].to_bytes()).to_bigint());
        let price = Felt::from_bytes_be(&event.data[3].to_bytes()).to_bigint();
        let pair_id = int_to_ascii(&Felt::from_bytes_be(&event.data[4].to_bytes()).to_bigint());
        let volume = Felt::from_bytes_be(&event.data[5].to_bytes()).to_bigint();

        Some(SpotEntry {
            timestamp: timestamp.to_string(),
            source: source.unwrap_or_default(),
            publisher: publisher.unwrap_or_default(),
            price: price.to_string(),
            pair_id: pair_id.unwrap_or_default(),
            volume: volume.to_string(),
            block_number,
        })
    }

    pub fn redis_key(&self) -> String {
        format!("spot:{}", self.pair_id)
    }
}

fn int_to_ascii(num: &BigInt) -> Option<String> {
    let bytes = num.to_bytes_be().1;
    String::from_utf8(bytes.into_iter().filter(|&byte| byte.is_ascii()).collect()).ok()
}
