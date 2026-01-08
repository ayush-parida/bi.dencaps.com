use mongodb::{Client, Database, Collection};
use redis::aio::ConnectionManager;
use std::sync::Arc;
use crate::models::{User, Project, AnalyticsQuery, Conversation};
use crate::config::Config;

#[derive(Clone)]
pub struct DatabaseManager {
    pub db: Database,
    pub redis: Arc<ConnectionManager>,
}

impl DatabaseManager {
    pub async fn new(config: &Config) -> Result<Self, String> {
        // MongoDB connection
        let client = Client::with_uri_str(&config.mongodb_uri)
            .await
            .map_err(|e| format!("Failed to connect to MongoDB: {}", e))?;
        
        let db = client.database(&config.mongodb_database);

        // Test MongoDB connection
        db.run_command(mongodb::bson::doc! { "ping": 1 })
            .await
            .map_err(|e| format!("MongoDB ping failed: {}", e))?;

        // Redis connection
        let redis_client = redis::Client::open(config.redis_uri.as_str())
            .map_err(|e| format!("Failed to create Redis client: {}", e))?;
        
        let redis = ConnectionManager::new(redis_client)
            .await
            .map_err(|e| format!("Failed to connect to Redis: {}", e))?;

        log::info!("Database connections established successfully");

        Ok(DatabaseManager {
            db,
            redis: Arc::new(redis),
        })
    }

    pub fn users_collection(&self) -> Collection<User> {
        self.db.collection("users")
    }

    pub fn projects_collection(&self) -> Collection<Project> {
        self.db.collection("projects")
    }

    pub fn queries_collection(&self) -> Collection<AnalyticsQuery> {
        self.db.collection("analytics_queries")
    }

    pub fn conversations_collection(&self) -> Collection<Conversation> {
        self.db.collection("conversations")
    }

    pub async fn create_indexes(&self) -> Result<(), String> {
        use mongodb::IndexModel;
        use mongodb::bson::doc;

        // User indexes
        let user_email_index = IndexModel::builder()
            .keys(doc! { "email": 1 })
            .options(mongodb::options::IndexOptions::builder()
                .unique(true)
                .build())
            .build();

        let user_tenant_index = IndexModel::builder()
            .keys(doc! { "tenant_id": 1 })
            .build();

        self.users_collection()
            .create_indexes(vec![user_email_index, user_tenant_index])
            .await
            .map_err(|e| format!("Failed to create user indexes: {}", e))?;

        // Project indexes
        let project_tenant_index = IndexModel::builder()
            .keys(doc! { "tenant_id": 1 })
            .build();

        let project_owner_index = IndexModel::builder()
            .keys(doc! { "owner_id": 1 })
            .build();

        self.projects_collection()
            .create_indexes(vec![project_tenant_index, project_owner_index])
            .await
            .map_err(|e| format!("Failed to create project indexes: {}", e))?;

        // Query indexes
        let query_project_index = IndexModel::builder()
            .keys(doc! { "project_id": 1 })
            .build();

        let query_user_index = IndexModel::builder()
            .keys(doc! { "user_id": 1 })
            .build();

        self.queries_collection()
            .create_indexes(vec![query_project_index, query_user_index])
            .await
            .map_err(|e| format!("Failed to create query indexes: {}", e))?;

        // Conversation indexes
        let conversation_project_index = IndexModel::builder()
            .keys(doc! { "project_id": 1 })
            .build();

        let conversation_user_index = IndexModel::builder()
            .keys(doc! { "user_id": 1 })
            .build();

        let conversation_id_index = IndexModel::builder()
            .keys(doc! { "conversation_id": 1 })
            .options(mongodb::options::IndexOptions::builder()
                .unique(true)
                .build())
            .build();

        self.conversations_collection()
            .create_indexes(vec![conversation_project_index, conversation_user_index, conversation_id_index])
            .await
            .map_err(|e| format!("Failed to create conversation indexes: {}", e))?;

        log::info!("Database indexes created successfully");
        Ok(())
    }
}
