use crate::db::DatabaseManager;
use crate::models::{Conversation, ChatMessage, ConversationResponse, ChatMessageResponse};
use crate::services::AIService;
use mongodb::bson::{doc, DateTime as BsonDateTime};
use redis::AsyncCommands;
use serde_json;
use uuid::Uuid;

pub struct ChatService {
    db_manager: DatabaseManager,
    ai_service: AIService,
}

impl ChatService {
    pub fn new(db_manager: DatabaseManager, ai_service: AIService) -> Self {
        ChatService {
            db_manager,
            ai_service,
        }
    }

    pub async fn send_message(
        &self,
        user_id: Uuid,
        project_id: Uuid,
        message: String,
        conversation_id: Option<Uuid>,
    ) -> Result<(String, ChatMessageResponse), String> {
        // Get or create conversation
        let conv_id = conversation_id.unwrap_or_else(Uuid::new_v4);
        
        let mut conversation = if conversation_id.is_some() {
            // Fetch existing conversation
            match self.get_conversation(&conv_id, &user_id).await? {
                Some(conv) => conv,
                None => return Err("Conversation not found".to_string()),
            }
        } else {
            // Create new conversation
            let title = if message.len() > 50 {
                format!("{}...", &message[..47])
            } else {
                message.clone()
            };

            Conversation {
                id: None,
                conversation_id: conv_id,
                project_id,
                user_id,
                title,
                messages: vec![],
                created_at: BsonDateTime::now(),
                updated_at: BsonDateTime::now(),
            }
        };

        // Add user message
        let user_message = ChatMessage {
            role: "user".to_string(),
            content: message.clone(),
            timestamp: BsonDateTime::now(),
        };
        conversation.messages.push(user_message);

        // Build context from conversation history
        let context = self.build_context(&conversation.messages);

        // Get AI response
        let ai_response = self
            .ai_service
            .process_chat_message(&message, context.as_deref())
            .await?;

        // Add AI message
        let ai_message = ChatMessage {
            role: "assistant".to_string(),
            content: ai_response.clone(),
            timestamp: BsonDateTime::now(),
        };
        conversation.messages.push(ai_message.clone());

        // Update conversation
        conversation.updated_at = BsonDateTime::now();

        // Save to database
        self.save_conversation(&conversation).await?;

        // Cache in Redis
        self.cache_conversation(&conversation).await.ok();

        Ok((
            conv_id.to_string(),
            ChatMessageResponse {
                role: "assistant".to_string(),
                content: ai_response,
                timestamp: conversation.updated_at.to_string(),
            },
        ))
    }

    pub async fn get_conversation(
        &self,
        conversation_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Option<Conversation>, String> {
        // Try to get from cache first
        if let Ok(Some(cached)) = self.get_cached_conversation(conversation_id).await {
            // Verify user has access
            if cached.user_id == *user_id {
                return Ok(Some(cached));
            }
        }

        // Get from database
        let collection = self.db_manager.conversations_collection();
        let filter = doc! {
            "conversation_id": conversation_id.to_string(),
            "user_id": user_id.to_string(),
        };

        let conversation = collection
            .find_one(filter)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        // Cache if found
        if let Some(ref conv) = conversation {
            self.cache_conversation(conv).await.ok();
        }

        Ok(conversation)
    }

    pub async fn get_project_conversations(
        &self,
        project_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Vec<ConversationResponse>, String> {
        let collection = self.db_manager.conversations_collection();
        let filter = doc! {
            "project_id": project_id.to_string(),
            "user_id": user_id.to_string(),
        };

        let mut cursor = collection
            .find(filter)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let mut conversations = Vec::new();
        use futures::StreamExt;
        while let Some(result) = cursor.next().await {
            match result {
                Ok(conv) => conversations.push(conv.into()),
                Err(e) => log::warn!("Failed to parse conversation: {}", e),
            }
        }

        Ok(conversations)
    }

    async fn save_conversation(&self, conversation: &Conversation) -> Result<(), String> {
        let collection = self.db_manager.conversations_collection();
        
        let filter = doc! { "conversation_id": conversation.conversation_id.to_string() };
        
        let options = mongodb::options::ReplaceOptions::builder()
            .upsert(true)
            .build();

        collection
            .replace_one(filter, conversation)
            .with_options(options)
            .await
            .map_err(|e| format!("Failed to save conversation: {}", e))?;

        Ok(())
    }

    async fn cache_conversation(&self, conversation: &Conversation) -> Result<(), String> {
        let mut redis = self.db_manager.redis.as_ref().clone();
        let key = format!("conversation:{}", conversation.conversation_id);
        let value = serde_json::to_string(conversation)
            .map_err(|e| format!("Failed to serialize conversation: {}", e))?;

        redis
            .set_ex::<_, _, ()>(key, value, 3600) // 1 hour TTL
            .await
            .map_err(|e| format!("Failed to cache conversation: {}", e))?;

        Ok(())
    }

    async fn get_cached_conversation(&self, conversation_id: &Uuid) -> Result<Option<Conversation>, String> {
        let mut redis = self.db_manager.redis.as_ref().clone();
        let key = format!("conversation:{}", conversation_id);

        let value: Option<String> = redis
            .get(key)
            .await
            .map_err(|e| format!("Failed to get cached conversation: {}", e))?;

        match value {
            Some(json) => {
                let conversation = serde_json::from_str(&json)
                    .map_err(|e| format!("Failed to deserialize conversation: {}", e))?;
                Ok(Some(conversation))
            }
            None => Ok(None),
        }
    }

    fn build_context(&self, messages: &[ChatMessage]) -> Option<String> {
        if messages.is_empty() {
            return None;
        }

        // Take last 10 messages for context (exclude the latest user message)
        let context_messages: Vec<String> = messages
            .iter()
            .rev()
            .skip(1)
            .take(10)
            .rev()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect();

        if context_messages.is_empty() {
            None
        } else {
            Some(context_messages.join("\n"))
        }
    }

    pub async fn check_rate_limit(&self, user_id: &Uuid) -> Result<bool, String> {
        let mut redis = self.db_manager.redis.as_ref().clone();
        let key = format!("chat_rate_limit:{}", user_id);

        // Get current count
        let count: Option<i32> = redis
            .get(&key)
            .await
            .map_err(|e| format!("Failed to check rate limit: {}", e))?;

        let current = count.unwrap_or(0);

        // Rate limit: 20 messages per minute
        if current >= 20 {
            return Ok(false);
        }

        // Increment counter
        let _: () = redis
            .incr(&key, 1)
            .await
            .map_err(|e| format!("Failed to increment rate limit: {}", e))?;

        // Set expiry if this is the first request
        if count.is_none() {
            let _: () = redis
                .expire(&key, 60)
                .await
                .map_err(|e| format!("Failed to set rate limit expiry: {}", e))?;
        }

        Ok(true)
    }
}
