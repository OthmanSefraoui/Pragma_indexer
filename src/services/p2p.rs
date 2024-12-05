// src/services/p2p.rs

use anyhow::Result;
use futures::StreamExt;
use libp2p::{
    core::{self, transport::Transport, upgrade},
    gossipsub::{self, IdentTopic, MessageAuthenticity, ValidationMode},
    identity, mdns, noise,
    swarm::{self, NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, StreamProtocol,
};
use num_bigint::BigInt;
use secp256k1::{Message, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TwapMessage {
    pub pair_id: String,
    pub twap: String,
    pub period: u64,
    pub signature: String,
    pub timestamp: u64,
    pub public_key: String,
}

// Define our network behaviour
#[derive(NetworkBehaviour)]
struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

pub struct P2PService {
    peer_id: PeerId,
    swarm: swarm::Swarm<MyBehaviour>,
    topics: Vec<IdentTopic>,
}

impl P2PService {
    pub async fn new(listen_addr: Multiaddr, bootstrap_peers: Vec<Multiaddr>) -> Result<Self> {
        // Create a random key for our identity
        let id_keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id_keys.public());
        println!("Local peer id: {:?}", peer_id);

        // Set up an encrypted TCP transport over yamux
        let tcp_transport = tcp::tokio::Transport::default();
        let noise_keys = identity::Keypair::generate_ed25519();

        // Use with_tokio_executor to ensure proper thread safety
        let transport = tcp_transport
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&noise_keys).unwrap())
            .multiplex(yamux::Config::default())
            .boxed();

        // Set up gossipsub with proper tokio compatibility
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(ValidationMode::Strict)
            .build()
            .expect("Valid gossipsub config");

        let gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(id_keys.clone()),
            gossipsub_config,
        )
        .expect("Valid gossipsub params");

        // Use tokio-specific mDNS implementation
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

        let behaviour = MyBehaviour { gossipsub, mdns };

        // Create a swarm with tokio executor
        let config = swarm::Config::with_tokio_executor();
        let mut swarm = swarm::Swarm::new(transport, behaviour, peer_id, config);

        // Listen on the provided address
        swarm.listen_on(listen_addr)?;

        let topics = vec![IdentTopic::new("twap-updates")];

        // Subscribe to topics
        for topic in &topics {
            swarm.behaviour_mut().gossipsub.subscribe(topic)?;
        }

        // Connect to bootstrap peers
        for addr in bootstrap_peers {
            swarm.dial(addr)?;
        }

        Ok(P2PService {
            peer_id,
            swarm,
            topics,
        })
    }

    fn handle_twap_message(&self, message: TwapMessage) -> Result<()> {
        // Verify the signature
        let secp = Secp256k1::new();
        let twap_bigint = BigInt::from(message.twap.parse::<i64>()?);
        let message_str = twap_bigint.to_string();

        let mut hasher = Sha256::new();
        hasher.update(message_str.as_bytes());
        let message_hash = hasher.finalize();
        let signature = secp256k1::ecdsa::Signature::from_der(&hex::decode(&message.signature)?)?;
        let public_key = secp256k1::PublicKey::from_slice(&hex::decode(&message.public_key)?)?;
        let message = Message::from_slice(&message_hash)?;

        match secp.verify_ecdsa(&message, &signature, &public_key) {
            Ok(_) => {
                println!("Valid signature from peer");
                // Process the verified message
                Ok(())
            }
            Err(e) => {
                println!("Invalid signature from peer: {}", e);
                Err(anyhow::anyhow!("Invalid signature"))
            }
        }
    }

    pub async fn run(
        mut self,
        mut message_receiver: mpsc::UnboundedReceiver<TwapMessage>,
    ) -> Result<()> {
        loop {
            tokio::select! {
                Some(message) = message_receiver.recv() => {
                    let data = serde_json::to_string(&message)?;
                    // Broadcast to all topics
                    for topic in &self.topics {
                        if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(
                            topic.clone(),
                            data.clone().as_bytes(),
                        ) {
                            println!("Publishing error: {}", e);
                        }
                    }
                }
                event = self.swarm.select_next_some() => {
                    match event {
                        SwarmEvent::Behaviour(behaviour) => {
                            match behaviour {
                                MyBehaviourEvent::Mdns(mdns::Event::Discovered(list)) => {
                                    for (peer_id, addr) in list {
                                        println!("mDNS discovered peer: {peer_id}, addr: {addr}");
                                        self.swarm.dial(addr)?;
                                    }
                                }
                                MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                                    propagation_source: peer_id,
                                    message_id: id,
                                    message,
                                }) => {
                                    println!(
                                        "Got message: '{}' with id: {} from peer: {:?}",
                                        String::from_utf8_lossy(&message.data),
                                        id,
                                        peer_id
                                    );

                                    if let Ok(twap_message) = serde_json::from_slice::<TwapMessage>(&message.data) {
                                        match self.handle_twap_message(twap_message.clone()) {
                                            Ok(_) => {
                                                let pair_id = twap_message.pair_id;
                                                println!(
                                                    "Verified and received TWAP update for pair_id: {}",
                                                    pair_id
                                                )
                                            },
                                            Err(e) => println!("Message verification failed: {}", e),
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }
}
