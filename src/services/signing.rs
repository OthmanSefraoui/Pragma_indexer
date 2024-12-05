use anyhow::{anyhow, Result};
use num_bigint::BigInt;
use secp256k1::{Message, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct SigningService {
    secp: Secp256k1<secp256k1::All>,
    secret_key: SecretKey,
    public_key: String,
}

impl SigningService {
    pub fn new(private_key_hex: &str) -> Result<Self> {
        let secp = Secp256k1::new();
        let clean_key = private_key_hex.trim_start_matches("0x");

        let secret_key = SecretKey::from_slice(&hex::decode(clean_key)?)
            .map_err(|e| anyhow!("Invalid private key: {}", e))?;
        let public_key =
            hex::encode(secp256k1::PublicKey::from_secret_key(&secp, &secret_key).serialize());

        Ok(SigningService {
            secp,
            secret_key,
            public_key,
        })
    }

    pub fn get_public_key(&self) -> &str {
        &self.public_key
    }

    pub fn sign_twap(&self, twap: f64) -> Result<String> {
        let twap_bigint = BigInt::from((twap) as u64);

        let twap_str = twap_bigint.to_string();

        let mut hasher = Sha256::new();
        hasher.update(twap_str.as_bytes());
        let message_hash = hasher.finalize();

        let message = Message::from_slice(&message_hash)?;

        let signature = self.secp.sign_ecdsa(&message, &self.secret_key);

        Ok(hex::encode(signature.serialize_der()))
    }
}
