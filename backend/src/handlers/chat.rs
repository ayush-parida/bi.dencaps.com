use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use serde::Serialize;
use validator::Validate;
use uuid::Uuid;
use crate::models::{SendMessageDto, ChatResponse, ConversationResponse, Permission};
use crate::services::{ChatService, RbacService};
use crate::utils::Claims;
use crate::middleware::check_permission;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
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
