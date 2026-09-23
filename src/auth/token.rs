use rand::RngExt;
use sha2::{Digest, Sha256};

pub fn generate_refresh_token() -> String {
    let bytes: [u8; 32] = rand::rng().random();
    return format!("vkr_{}", hex::encode(bytes));
}

pub fn hash_refresh_token(raw_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    hex::encode(hasher.finalize())
}

