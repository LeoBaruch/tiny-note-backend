use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub jti: String,
}

pub fn generate_token(user_id: Uuid, secret: &str, ttl_minutes: i64) -> anyhow::Result<(String, Claims)> {
    let exp = Utc::now() + Duration::minutes(ttl_minutes);
    let claims = Claims {
        sub: user_id,
        exp: exp.timestamp() as usize,
        jti: Uuid::new_v4().to_string(),
    };
    let token = encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret.as_bytes()))?;
    Ok((token, claims))
}

pub fn validate_token(token: &str, secret: &str) -> anyhow::Result<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)?;
    Ok(data.claims)
}