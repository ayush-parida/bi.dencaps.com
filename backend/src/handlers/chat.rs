use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use actix_web::http::header;
use serde::{Serialize, Deserialize};
use validator::Validate;
use uuid::Uuid;
use futures::StreamExt;
use crate::models::{SendMessageDto, ChatResponse, ConversationResponse, Permission};
use crate::services::{ChatService, RbacService};
use crate::utils::Claims;
use crate::middleware::check_permission;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

/// SSE event wrapper for streaming responses
#[derive(Debug, Serialize)]
struct StreamEvent {
    event: String,
    data: String,
}

/// Request DTO for streaming messages
#[derive(Debug, Deserialize, Validate)]
pub struct StreamMessageDto {
    #[validate(length(min = 1, max = 10000))]
    pub message: String,
    pub project_id: String,
    pub conversation_id: Option<String>,
}

/// Request DTO for saving the assistant response after streaming
#[derive(Debug, Deserialize, Validate)]
pub struct SaveStreamedResponseDto {
    pub conversation_id: String,
    #[validate(length(min = 1))]
    pub content: String,
}

/// Request DTO for regenerating a response
#[derive(Debug, Deserialize, Validate)]
pub struct RegenerateMessageDto {
    pub conversation_id: String,
    pub from_index: usize,
}

pub async fn send_message(
    chat_service: web::Data<ChatService>,
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
    dto: web::Json<SendMessageDto>,
) -> HttpResponse {
    // Validate input
    if let Err(e) = dto.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Validation error: {}", e),
        });
    }

    // Get user from JWT claims
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
            });
        }
    };

    // Parse user_id from claims
    let user_id = match Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user_id".to_string(),
            });
        }
    };

    // Check permission to write to chat in this project
    if let Err(e) = check_permission(
        &rbac_service,
        &claims.user_id,
        Some(&dto.project_id),
        Permission::ChatWrite
    ).await {
        return HttpResponse::Forbidden().json(ErrorResponse { error: e.to_string() });
    }

    // Parse project_id
    let project_id = match Uuid::parse_str(&dto.project_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid project_id format".to_string(),
            });
        }
    };

    // Check rate limit
    match chat_service.check_rate_limit(&user_id).await {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::TooManyRequests().json(ErrorResponse {
                error: "Rate limit exceeded. Please wait before sending more messages.".to_string(),
            });
        }
        Err(e) => {
            log::error!("Rate limit check failed: {}", e);
            // If Redis fails, deny the request to maintain security
            return HttpResponse::ServiceUnavailable().json(ErrorResponse {
                error: "Rate limiting service temporarily unavailable. Please try again later.".to_string(),
            });
        }
    }

    // Parse conversation_id if provided
    let conversation_id = if let Some(ref conv_id_str) = dto.conversation_id {
        match Uuid::parse_str(conv_id_str) {
            Ok(id) => Some(id),
            Err(_) => {
                return HttpResponse::BadRequest().json(ErrorResponse {
                    error: "Invalid conversation_id format".to_string(),
                });
            }
        }
    } else {
        None
    };

    // Send message
    match chat_service
        .send_message(
            user_id,
            project_id,
            dto.message.clone(),
            conversation_id,
        )
        .await
    {
        Ok((conv_id, message)) => HttpResponse::Ok().json(ChatResponse {
            conversation_id: conv_id,
            message,
        }),
        Err(e) => {
            log::error!("Failed to process chat message: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to process message: {}", e),
            })
        }
    }
}

pub async fn get_conversation(
    chat_service: web::Data<ChatService>,
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    // Get user from JWT claims
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
            });
        }
    };

    // Parse user_id from claims
    let user_id = match Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user_id".to_string(),
            });
        }
    };

    // Parse conversation_id
    let conversation_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid conversation_id format".to_string(),
            });
        }
    };

    // Get conversation first to check project_id
    match chat_service.get_conversation(&conversation_id, &user_id).await {
        Ok(Some(conversation)) => {
            // Check permission using RBAC for the project this conversation belongs to
            if let Err(e) = check_permission(
                &rbac_service,
                &claims.user_id,
                Some(&conversation.project_id.to_string()),
                Permission::ChatRead
            ).await {
                return HttpResponse::Forbidden().json(ErrorResponse { error: e.to_string() });
            }
            
            let response: ConversationResponse = conversation.into();
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Conversation not found".to_string(),
        }),
        Err(e) => {
            log::error!("Failed to get conversation: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to get conversation: {}", e),
            })
        }
    }
}

pub async fn delete_conversation(
    chat_service: web::Data<ChatService>,
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    // Get user from JWT claims
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
            });
        }
    };

    // Parse user_id from claims
    let user_id = match Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user_id".to_string(),
            });
        }
    };

    // Parse conversation_id
    let conversation_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid conversation_id format".to_string(),
            });
        }
    };

    // Get conversation first to check project_id for permission
    let conversation = match chat_service.get_conversation(&conversation_id, &user_id).await {
        Ok(Some(conv)) => conv,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Conversation not found".to_string(),
            });
        }
        Err(e) => {
            log::error!("Failed to get conversation: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to get conversation: {}", e),
            });
        }
    };

    // Check permission using RBAC for the project this conversation belongs to
    if let Err(e) = check_permission(
        &rbac_service,
        &claims.user_id,
        Some(&conversation.project_id.to_string()),
        Permission::ChatDelete
    ).await {
        return HttpResponse::Forbidden().json(ErrorResponse { error: e.to_string() });
    }

    // Delete the conversation
    match chat_service.delete_conversation(&conversation_id, &user_id).await {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({
            "success": true
        })),
        Ok(false) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Conversation not found".to_string(),
        }),
        Err(e) => {
            log::error!("Failed to delete conversation: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to delete conversation: {}", e),
            })
        }
    }
}

pub async fn get_project_conversations(
    chat_service: web::Data<ChatService>,
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    // Get user from JWT claims
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
            });
        }
    };

    // Parse user_id from claims
    let user_id = match Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user_id".to_string(),
            });
        }
    };

    let project_id_str = path.into_inner();

    // Check permission to read chat in this project
    if let Err(e) = check_permission(
        &rbac_service,
        &claims.user_id,
        Some(&project_id_str),
        Permission::ChatRead
    ).await {
        return HttpResponse::Forbidden().json(ErrorResponse { error: e.to_string() });
    }

    // Parse project_id
    let project_id = match Uuid::parse_str(&project_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid project_id format".to_string(),
            });
        }
    };

    // Get conversations
    match chat_service
        .get_project_conversations(&project_id, &user_id)
        .await
    {
        Ok(conversations) => HttpResponse::Ok().json(conversations),
        Err(e) => {
            log::error!("Failed to get project conversations: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to get conversations: {}", e),
            })
        }
    }
}

/// Get lightweight conversation summaries for a project (without message content)
pub async fn get_project_conversation_summaries(
    chat_service: web::Data<ChatService>,
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    // Get user from JWT claims
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
            });
        }
    };

    // Parse user_id from claims
    let user_id = match Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user_id".to_string(),
            });
        }
    };

    let project_id_str = path.into_inner();

    // Check permission to read chat in this project
    if let Err(e) = check_permission(
        &rbac_service,
        &claims.user_id,
        Some(&project_id_str),
        Permission::ChatRead
    ).await {
        return HttpResponse::Forbidden().json(ErrorResponse { error: e.to_string() });
    }

    // Parse project_id
    let project_id = match Uuid::parse_str(&project_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid project_id format".to_string(),
            });
        }
    };

    // Get conversation summaries
    match chat_service
        .get_project_conversation_summaries(&project_id, &user_id)
        .await
    {
        Ok(summaries) => HttpResponse::Ok().json(summaries),
        Err(e) => {
            log::error!("Failed to get project conversation summaries: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to get conversations: {}", e),
            })
        }
    }
}

/// Streaming chat endpoint - returns Server-Sent Events (SSE)
pub async fn stream_message(
    chat_service: web::Data<ChatService>,
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
    dto: web::Json<StreamMessageDto>,
) -> HttpResponse {
    // Validate input
    if let Err(e) = dto.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Validation error: {}", e),
        });
    }

    // Get user from JWT claims
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
            });
        }
    };

    // Parse user_id from claims
    let user_id = match Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user_id".to_string(),
            });
        }
    };

    // Check permission
    if let Err(e) = check_permission(
        &rbac_service,
        &claims.user_id,
        Some(&dto.project_id),
        Permission::ChatWrite
    ).await {
        return HttpResponse::Forbidden().json(ErrorResponse { error: e.to_string() });
    }

    // Parse project_id
    let project_id = match Uuid::parse_str(&dto.project_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid project_id format".to_string(),
            });
        }
    };

    // Check rate limit
    match chat_service.check_rate_limit(&user_id).await {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::TooManyRequests().json(ErrorResponse {
                error: "Rate limit exceeded. Please wait before sending more messages.".to_string(),
            });
        }
        Err(e) => {
            log::error!("Rate limit check failed: {}", e);
            return HttpResponse::ServiceUnavailable().json(ErrorResponse {
                error: "Rate limiting service temporarily unavailable.".to_string(),
            });
        }
    }

    // Parse conversation_id if provided
    let conversation_id = if let Some(ref conv_id_str) = dto.conversation_id {
        match Uuid::parse_str(conv_id_str) {
            Ok(id) => Some(id),
            Err(_) => {
                return HttpResponse::BadRequest().json(ErrorResponse {
                    error: "Invalid conversation_id format".to_string(),
                });
            }
        }
    } else {
        None
    };

    // Get streaming response
    let (conv_id, stream) = match chat_service
        .stream_message(user_id, project_id, dto.message.clone(), conversation_id)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            log::error!("Failed to start streaming: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to start streaming: {}", e),
            });
        }
    };

    // Create the SSE response stream
    let conv_id_clone = conv_id.clone();
    let response_stream = async_stream::stream! {
        // Send conversation_id as first event
        let init_event = format!("event: init\ndata: {}\n\n", serde_json::json!({
            "conversation_id": conv_id_clone
        }));
        yield Ok::<_, actix_web::error::Error>(web::Bytes::from(init_event));

        // Stream the AI response chunks
        let mut pinned_stream = stream;
        while let Some(chunk_result) = pinned_stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    // Forward the SSE data directly from the RAG API
                    yield Ok(web::Bytes::from(bytes.to_vec()));
                }
                Err(e) => {
                    log::error!("Stream error: {}", e);
                    let error_event = format!("event: error\ndata: {}\n\n", serde_json::json!({
                        "error": e.to_string()
                    }));
                    yield Ok(web::Bytes::from(error_event));
                    break;
                }
            }
        }

        // Send done event
        let done_event = "event: done\ndata: {}\n\n".to_string();
        yield Ok(web::Bytes::from(done_event));
    };

    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "text/event-stream"))
        .insert_header((header::CACHE_CONTROL, "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(response_stream)
}

/// Save the assistant response after streaming is complete
pub async fn save_streamed_response(
    chat_service: web::Data<ChatService>,
    req: HttpRequest,
    dto: web::Json<SaveStreamedResponseDto>,
) -> HttpResponse {
    // Validate input
    if let Err(e) = dto.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Validation error: {}", e),
        });
    }

    // Get user from JWT claims
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
            });
        }
    };

    // Parse user_id from claims
    let user_id = match Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user_id".to_string(),
            });
        }
    };

    // Parse conversation_id
    let conversation_id = match Uuid::parse_str(&dto.conversation_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid conversation_id format".to_string(),
            });
        }
    };

    // Save the assistant message
    match chat_service
        .append_assistant_message(&conversation_id, &user_id, dto.content.clone())
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true
        })),
        Err(e) => {
            log::error!("Failed to save streamed response: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to save response: {}", e),
            })
        }
    }
}

/// Regenerate a response from a specific message index
/// This removes messages from the index onwards and regenerates
pub async fn regenerate_message_stream(
    chat_service: web::Data<ChatService>,
    req: HttpRequest,
    dto: web::Json<RegenerateMessageDto>,
) -> HttpResponse {
    // Validate input
    if let Err(e) = dto.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Validation error: {}", e),
        });
    }

    // Get user from JWT claims
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
            });
        }
    };

    // Parse user_id from claims
    let user_id = match Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user_id".to_string(),
            });
        }
    };

    // Parse conversation_id
    let conversation_id = match Uuid::parse_str(&dto.conversation_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid conversation_id format".to_string(),
            });
        }
    };

    // Get streaming response with regeneration
    let (conv_id, stream) = match chat_service
        .regenerate_from_index(user_id, conversation_id, dto.from_index)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            log::error!("Failed to start regeneration: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to start regeneration: {}", e),
            });
        }
    };

    // Create the SSE response stream
    let conv_id_clone = conv_id.clone();
    let response_stream = async_stream::stream! {
        // Send conversation_id as first event
        let init_event = format!("event: init\ndata: {}\n\n", serde_json::json!({
            "conversation_id": conv_id_clone
        }));
        yield Ok::<_, actix_web::error::Error>(web::Bytes::from(init_event));

        // Stream the AI response chunks
        let mut pinned_stream = stream;
        while let Some(chunk_result) = pinned_stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    // Forward the SSE data directly from the RAG API
                    yield Ok(web::Bytes::from(bytes.to_vec()));
                }
                Err(e) => {
                    log::error!("Stream error: {}", e);
                    let error_event = format!("event: error\ndata: {}\n\n", serde_json::json!({
                        "error": e.to_string()
                    }));
                    yield Ok(web::Bytes::from(error_event));
                    break;
                }
            }
        }

        // Send done event
        let done_event = "event: done\ndata: {}\n\n".to_string();
        yield Ok(web::Bytes::from(done_event));
    };

    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "text/event-stream"))
        .insert_header((header::CACHE_CONTROL, "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(response_stream)
}
