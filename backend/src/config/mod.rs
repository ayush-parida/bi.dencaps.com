use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum AIProvider {
    /// LM Studio - for local development and testing (OpenAI-compatible API)
    LMStudio,
    /// OpenAI - for production use
    OpenAI,
    /// Custom RAG API - for production use with custom endpoint
    CustomRAG,
}

impl AIProvider {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "openai" => AIProvider::OpenAI,
            "lmstudio" | "lm_studio" | "lm-studio" => AIProvider::LMStudio,
            "custom" | "custom_rag" | "customrag" | "rag" => AIProvider::CustomRAG,
            _ => AIProvider::LMStudio, // Default to LMStudio for development
        }
    }
}

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
    // AI Provider Configuration
    pub ai_provider: AIProvider,
    pub ai_api_url: String,
    pub ai_model_name: String,
    pub ai_api_key: Option<String>,
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

        // AI Provider configuration - supports LMStudio (dev), OpenAI (prod), CustomRAG (prod)
        let ai_provider = AIProvider::from_str(
            &env::var("AI_PROVIDER").unwrap_or_else(|_| "lmstudio".to_string())
        );
        
        // Get API URL based on provider or use unified AI_API_URL
        let ai_api_url = env::var("AI_API_URL")
            .or_else(|_| env::var("LM_STUDIO_API_URL"))
            .or_else(|_| env::var("OPENAI_API_URL"))
            .unwrap_or_else(|_| match ai_provider {
                AIProvider::OpenAI => "https://api.openai.com".to_string(),
                AIProvider::LMStudio => "http://localhost:1234".to_string(),
                AIProvider::CustomRAG => "http://localhost:8001".to_string(),
            });
        
        // Get model name based on provider or use unified AI_MODEL_NAME
        // Note: CustomRAG doesn't use model_name but we keep it for consistency
        let ai_model_name = env::var("AI_MODEL_NAME")
            .or_else(|_| env::var("LM_STUDIO_MODEL_NAME"))
            .or_else(|_| env::var("OPENAI_MODEL_NAME"))
            .unwrap_or_else(|_| match ai_provider {
                AIProvider::OpenAI => "gpt-4".to_string(),
                AIProvider::LMStudio => "GPT-OSS-20B".to_string(),
                AIProvider::CustomRAG => "default".to_string(),
            });
        
        // API Key (required for OpenAI and CustomRAG, optional for LMStudio)
        let ai_api_key = env::var("AI_API_KEY")
            .or_else(|_| env::var("OPENAI_API_KEY"))
            .ok();

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
            .unwrap_or_else(|_| "http://localhost:4202".to_string())
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
            ai_provider,
            ai_api_url,
            ai_model_name,
            ai_api_key,
            rate_limit_requests,
            rate_limit_window_secs,
            chat_rate_limit_messages,
            chat_rate_limit_window_secs,
            chat_context_message_limit,
            cors_allowed_origins,
        })
    }
}
