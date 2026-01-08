use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub mongodb_uri: String,
    pub mongodb_database: String,
    pub redis_uri: String,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub jwt_refresh_expiration: i64,
    pub lm_studio_api_url: String,
    pub lm_studio_model_name: String,
    pub rate_limit_requests: usize,
    pub rate_limit_window_secs: u64,
    pub chat_rate_limit_messages: usize,
    pub chat_rate_limit_window_secs: u64,
    pub chat_context_message_limit: usize,
    pub cors_allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        // Only load .env file in development (when RUST_ENV is not production)
        if std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) != "production" {
            dotenv::dotenv().ok();
        }

        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|_| "Invalid SERVER_PORT")?;

        let mongodb_uri = env::var("MONGODB_URI")
            .map_err(|_| "MONGODB_URI must be set")?;
        let mongodb_database = env::var("MONGODB_DATABASE")
            .map_err(|_| "MONGODB_DATABASE must be set")?;

        let redis_uri = env::var("REDIS_URI")
            .map_err(|_| "REDIS_URI must be set")?;

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET must be set")?;
        let jwt_expiration = env::var("JWT_EXPIRATION")
            .unwrap_or_else(|_| "86400".to_string())
            .parse::<i64>()
            .map_err(|_| "Invalid JWT_EXPIRATION")?;
        let jwt_refresh_expiration = env::var("JWT_REFRESH_EXPIRATION")
            .unwrap_or_else(|_| "2592000".to_string())
            .parse::<i64>()
            .map_err(|_| "Invalid JWT_REFRESH_EXPIRATION")?;

        let lm_studio_api_url = env::var("LM_STUDIO_API_URL")
            .map_err(|_| "LM_STUDIO_API_URL must be set")?;
        let lm_studio_model_name = env::var("LM_STUDIO_MODEL_NAME")
            .unwrap_or_else(|_| "GPT-OSS-20B".to_string());

        let rate_limit_requests = env::var("RATE_LIMIT_REQUESTS")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<usize>()
            .map_err(|_| "Invalid RATE_LIMIT_REQUESTS")?;
        let rate_limit_window_secs = env::var("RATE_LIMIT_WINDOW_SECS")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u64>()
            .map_err(|_| "Invalid RATE_LIMIT_WINDOW_SECS")?;

        let chat_rate_limit_messages = env::var("CHAT_RATE_LIMIT_MESSAGES")
            .unwrap_or_else(|_| "20".to_string())
            .parse::<usize>()
            .map_err(|_| "Invalid CHAT_RATE_LIMIT_MESSAGES")?;
        let chat_rate_limit_window_secs = env::var("CHAT_RATE_LIMIT_WINDOW_SECS")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u64>()
            .map_err(|_| "Invalid CHAT_RATE_LIMIT_WINDOW_SECS")?;
        let chat_context_message_limit = env::var("CHAT_CONTEXT_MESSAGE_LIMIT")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<usize>()
            .map_err(|_| "Invalid CHAT_CONTEXT_MESSAGE_LIMIT")?;

        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:4200".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Config {
            server_host,
            server_port,
            mongodb_uri,
            mongodb_database,
            redis_uri,
            jwt_secret,
            jwt_expiration,
            jwt_refresh_expiration,
            lm_studio_api_url,
            lm_studio_model_name,
            rate_limit_requests,
            rate_limit_window_secs,
            chat_rate_limit_messages,
            chat_rate_limit_window_secs,
            chat_context_message_limit,
            cors_allowed_origins,
        })
    }
}
