use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub tenant_id: String,
    pub exp: i64,
    pub iat: i64,
}

pub struct JwtManager {
    secret: String,
    expiration: i64,
    refresh_expiration: i64,
}

impl JwtManager {
    pub fn new(secret: String, expiration: i64, refresh_expiration: i64) -> Self {
        JwtManager {
            secret,
            expiration,
            refresh_expiration,
        }
    }

    pub fn generate_token(
        &self,
        user_id: &Uuid,
        email: &str,
        role: &str,
        tenant_id: &str,
        is_refresh: bool,
    ) -> Result<String, String> {
        let now = Utc::now().timestamp();
        let exp = if is_refresh {
            now + self.refresh_expiration
        } else {
            now + self.expiration
        };

        let claims = Claims {
            sub: user_id.to_string(),
            user_id: user_id.to_string(),
            email: email.to_string(),
            role: role.to_string(),
            tenant_id: tenant_id.to_string(),
            exp,
            iat: now,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| format!("Failed to generate token: {}", e))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, String> {
        let validation = Validation::new(Algorithm::HS256);
        
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|e| format!("Invalid token: {}", e))
    }
}

pub fn hash_password(password: &str) -> Result<String, String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| format!("Failed to hash password: {}", e))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    bcrypt::verify(password, hash)
        .map_err(|e| format!("Failed to verify password: {}", e))
}
