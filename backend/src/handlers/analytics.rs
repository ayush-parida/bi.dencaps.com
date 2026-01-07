use actix_web::{web, HttpResponse, HttpRequest};
use serde::Serialize;
use validator::Validate;
use crate::models::CreateQueryDto;
use crate::services::{AnalyticsService, ProjectService};
use crate::utils::Claims;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn create_query(
    analytics_service: web::Data<AnalyticsService>,
    project_service: web::Data<ProjectService>,
    dto: web::Json<CreateQueryDto>,
    req: HttpRequest,
) -> HttpResponse {
    if let Err(e) = dto.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Validation error: {}", e),
        });
    }

    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
            });
        }
    };

    let user_id = match uuid::Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user ID".to_string(),
            });
        }
    };

    // Check project access
    let project_id = match uuid::Uuid::parse_str(&dto.project_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid project ID".to_string(),
            });
        }
    };

    match project_service.check_user_access(&project_id, &user_id).await {
        Ok(has_access) => {
            if !has_access {
                return HttpResponse::Forbidden().json(ErrorResponse {
                    error: "Access denied to this project".to_string(),
                });
            }
        }
        Err(e) => {
            return HttpResponse::NotFound().json(ErrorResponse { error: e });
        }
    }

    match analytics_service.create_query(dto.into_inner(), &user_id).await {
        Ok(query) => HttpResponse::Created().json(query),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse { error: e }),
    }
}

pub async fn process_query(
    analytics_service: web::Data<AnalyticsService>,
    query_id: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
            });
        }
    };

    let query_uuid = match uuid::Uuid::parse_str(&query_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid query ID".to_string(),
            });
        }
    };

    match analytics_service.process_query(&query_uuid).await {
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "query_id": query_id.to_string(),
            "response": response,
        })),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: e }),
    }
}

pub async fn get_query_by_id(
    analytics_service: web::Data<AnalyticsService>,
    project_service: web::Data<ProjectService>,
    query_id: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
            });
        }
    };

    let query_uuid = match uuid::Uuid::parse_str(&query_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid query ID".to_string(),
            });
        }
    };

    let user_id = match uuid::Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user ID".to_string(),
            });
        }
    };

    let query = match analytics_service.get_query_by_id(&query_uuid).await {
        Ok(q) => q,
        Err(e) => {
            return HttpResponse::NotFound().json(ErrorResponse { error: e });
        }
    };

    // Check project access
    match project_service.check_user_access(&query.project_id, &user_id).await {
        Ok(has_access) => {
            if !has_access {
                return HttpResponse::Forbidden().json(ErrorResponse {
                    error: "Access denied to this query".to_string(),
                });
            }
        }
        Err(e) => {
            return HttpResponse::NotFound().json(ErrorResponse { error: e });
        }
    }

    HttpResponse::Ok().json(query)
}

pub async fn get_project_queries(
    analytics_service: web::Data<AnalyticsService>,
    project_service: web::Data<ProjectService>,
    project_id: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
            });
        }
    };

    let project_uuid = match uuid::Uuid::parse_str(&project_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid project ID".to_string(),
            });
        }
    };

    let user_id = match uuid::Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user ID".to_string(),
            });
        }
    };

    // Check project access
    match project_service.check_user_access(&project_uuid, &user_id).await {
        Ok(has_access) => {
            if !has_access {
                return HttpResponse::Forbidden().json(ErrorResponse {
                    error: "Access denied to this project".to_string(),
                });
            }
        }
        Err(e) => {
            return HttpResponse::NotFound().json(ErrorResponse { error: e });
        }
    }

    match analytics_service.get_project_queries(&project_uuid).await {
        Ok(queries) => HttpResponse::Ok().json(queries),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: e }),
    }
}
