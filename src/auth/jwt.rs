use jsonwebtoken::{
    decode, encode, errors::Error, get_current_timestamp, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const ACCESS_TOKEN_DURATION_SECS: u64 = 15*60;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub tid: Uuid,
    pub role: String,
    pub exp: u64,
    pub iat: u64,
}

pub fn create_access_token(user_id: Uuid, tenant_id: Uuid, role: &str, secret: &[u8]) -> Result<String, Error> {
    let now = get_current_timestamp();
    let claims = Claims {
        sub: user_id,
        tid: tenant_id, 
        role: role.to_string(),
        iat: now,
        exp: now+ACCESS_TOKEN_DURATION_SECS,
    };

    let header = Header::new(Algorithm::HS256);
    encode(&header, &claims, &EncodingKey::from_secret(secret))
}

pub fn verify_access_token(token: &str, secret: &[u8]) -> Result<Claims, Error> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 0;
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)?;
    Ok(token_data.claims)
}