use mongodb::bson::{doc, DateTime};
use uuid::Uuid;
use crate::db::DatabaseManager;
use crate::models::{AnalyticsQuery, CreateQueryDto, QueryStatus};
use crate::services::AIService;

pub struct AnalyticsService {
    db: DatabaseManager,
    ai_service: AIService,
}

impl AnalyticsService {
    pub fn new(db: DatabaseManager, ai_service: AIService) -> Self {
        AnalyticsService { db, ai_service }
    }

    pub async fn create_query(
        &self,
        dto: CreateQueryDto,
        user_id: &Uuid,
    ) -> Result<AnalyticsQuery, String> {
        // Validate project ID format
        Uuid::parse_str(&dto.project_id)
            .map_err(|_| "Invalid project ID format".to_string())?;

        let now = DateTime::now();

        let query = AnalyticsQuery {
            id: None,
            query_id: Uuid::new_v4().to_string(),
            project_id: dto.project_id,
            user_id: user_id.to_string(),
            query_text: dto.query_text,
            response_text: None,
            status: QueryStatus::Pending,
            created_at: now,
            completed_at: None,
        };

        self.db
            .queries_collection()
            .insert_one(&query)
            .await
            .map_err(|e| format!("Failed to create query: {}", e))?;

        Ok(query)
    }

    pub async fn process_query(&self, query_id: &Uuid) -> Result<String, String> {
        let uuid_str = query_id.to_string();
        
        let query = self.db
            .queries_collection()
            .find_one(doc! { "query_id": &uuid_str })
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Query not found".to_string())?;

        // Update status to processing
        self.db
            .queries_collection()
            .update_one(
                doc! { "query_id": &uuid_str },
                doc! { "$set": { "status": "Processing" } }
            )
            .await
            .map_err(|e| format!("Failed to update query status: {}", e))?;

        // Process with AI
        let response = match self.ai_service.process_analytics_query(&query.query_text, None).await {
            Ok(resp) => resp,
            Err(e) => {
                // Update status to failed
                self.db
                    .queries_collection()
                    .update_one(
                        doc! { "query_id": &uuid_str },
                        doc! { "$set": { 
                            "status": "Failed",
                            "response_text": format!("Error: {}", e),
                            "completed_at": DateTime::now()
                        } }
                    )
                    .await
                    .map_err(|e| format!("Failed to update query: {}", e))?;
                
                return Err(e);
            }
        };

        // Update with response
        self.db
            .queries_collection()
            .update_one(
                doc! { "query_id": &uuid_str },
                doc! { "$set": { 
                    "status": "Completed",
                    "response_text": &response,
                    "completed_at": DateTime::now()
                } }
            )
            .await
            .map_err(|e| format!("Failed to update query: {}", e))?;

        Ok(response)
    }

    pub async fn get_query_by_id(&self, query_id: &Uuid) -> Result<AnalyticsQuery, String> {
        let uuid_str = query_id.to_string();
        
        self.db
            .queries_collection()
            .find_one(doc! { "query_id": uuid_str })
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Query not found".to_string())
    }

    pub async fn get_project_queries(&self, project_id: &Uuid) -> Result<Vec<AnalyticsQuery>, String> {
        use futures::stream::TryStreamExt;
        
        let uuid_str = project_id.to_string();

        let cursor = self.db
            .queries_collection()
            .find(doc! { "project_id": uuid_str })
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        cursor
            .try_collect()
            .await
            .map_err(|e| format!("Failed to fetch queries: {}", e))
    }
}
