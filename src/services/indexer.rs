use core::time;

use crate::services::redis_client::RedisClient;
use crate::types::spot_entry::SpotEntry;
use anyhow::Result;
use apibara_core::starknet::v1alpha2::Event;
use apibara_core::{
    node::v1alpha2::DataFinality,
    starknet::v1alpha2::{Block, FieldElement, Filter, HeaderFilter},
};
use apibara_sdk::{configuration, ClientBuilder, Configuration, Uri};
use futures_util::TryStreamExt;
use num_bigint::BigInt;
use starknet::core::types::Felt;

const INDEXING_STREAM_CHUNK_SIZE: usize = 1;

pub struct Indexer {
    uri: Uri,
    apibara_api_key: String,
    from_block: u64,
    contract_address: String,
    redis_client: RedisClient,
}

impl Indexer {
    pub fn new(
        apibara_api_key: String,
        from_block: u64,
        contract_address: String,
        redis_url: String,
    ) -> Indexer {
        let uri = Uri::from_static("https://sepolia.starknet.a5a.ch");
        let redis_client = RedisClient::new(&redis_url).unwrap();
        Indexer {
            uri,
            apibara_api_key,
            from_block,
            contract_address,
            redis_client,
        }
    }

    /// Start indexing SubmittedSpotEntry events
    pub async fn run(&self) -> Result<()> {
        let stream_config = Configuration::<Filter>::default()
            .with_starting_block(self.from_block)
            .with_finality(DataFinality::DataStatusPending)
            .with_filter(|mut filter| {
                filter
                    .with_header(HeaderFilter::weak())
                    .add_event(|event| {
                        event
                            .with_from_address(
                                FieldElement::from_hex(&self.contract_address).unwrap(),
                            )
                            .with_keys(vec![FieldElement::from_hex(
                                "0x280bb2099800026f90c334a3a23888ffe718a2920ffbbf4f44c6d3d5efb613c",
                            )
                            .unwrap()])
                    })
                    .build()
            });

        let (config_client, config_stream) = configuration::channel(INDEXING_STREAM_CHUNK_SIZE);
        config_client.send(stream_config.clone()).await?;

        let mut stream = ClientBuilder::default()
            .with_bearer_token(Some(self.apibara_api_key.clone()))
            .connect(self.uri.clone())
            .await
            .unwrap()
            .start_stream::<Filter, Block, _>(config_stream)
            .await
            .unwrap();

        println!("🔍 Started indexing from block {}", self.from_block);

        loop {
            match stream.try_next().await {
                Ok(Some(response)) => {
                    if let apibara_sdk::DataMessage::Data { batch, .. } = response {
                        for block in batch {
                            for event in block.events {
                                if let Some(event) = event.event {
                                    let block_number =
                                        block.header.clone().map(|h| h.block_number).unwrap_or(0);

                                    self.handle_event(block_number, event).await?;
                                }
                            }
                        }
                    }
                }
                Ok(None) => continue,
                Err(e) => {
                    println!("Error while streaming: {}", e);
                    return Err(anyhow::anyhow!("Streaming error: {}", e));
                }
            }
        }
    }

    async fn handle_event(&self, block_number: u64, event: Event) -> Result<()> {
        if event.from_address.is_none() || event.data.is_empty() {
            return Ok(());
        }

        if let Some(entry) = SpotEntry::from_event(&event, block_number) {
            // println!("Block: {}", block_number);
            // println!("Storing entry for pair: {}", entry.pair_id);

            // Store in Redis
            self.redis_client.store_spot_entry(&entry).await?;
        }
        Ok(())
    }
}