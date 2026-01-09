use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use serde::Serialize;
use validator::Validate;
use crate::models::{CreateQueryDto, Permission};
use crate::services::{AnalyticsService, RbacService};
use crate::utils::Claims;
use crate::middleware::check_permission;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn create_query(
    analytics_service: web::Data<AnalyticsService>,
    rbac_service: web::Data<RbacService>,
    dto: web::Json<CreateQueryDto>,
    req: HttpRequest,
) -> HttpResponse {
    if let Err(e) = dto.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Validation error: {}", e),
        });
    }

    let extensions = req.extensions();
    let claims = match extensions.get::<Claims>() {
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

    // Check permission to create reports in this project
    if let Err(e) = check_permission(
        &rbac_service,
        &claims.user_id,
        Some(&dto.project_id),
        Permission::ReportCreate
    ).await {
        return HttpResponse::Forbidden().json(ErrorResponse { error: e.to_string() });
    }

    match analytics_service.create_query(dto.into_inner(), &user_id).await {
        Ok(query) => HttpResponse::Created().json(query),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse { error: e }),
    }
}

pub async fn process_query(
    analytics_service: web::Data<AnalyticsService>,
    rbac_service: web::Data<RbacService>,
    query_id: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let extensions = req.extensions();
    let claims = match extensions.get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
            });
        }
    };

    let query_uuid = match uuid::Uuid::parse_str(&query_id.as_str()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid query ID".to_string(),
            });
        }
    };

    // Get the query first to check project_id
    let query = match analytics_service.get_query_by_id(&query_uuid).await {
        Ok(q) => q,
        Err(e) => {
            return HttpResponse::NotFound().json(ErrorResponse { error: e });
        }
    };

    // Check permission to read reports in this project
    if let Err(e) = check_permission(
        &rbac_service,
        &claims.user_id,
        Some(&query.project_id),
        Permission::ReportRead
    ).await {
        return HttpResponse::Forbidden().json(ErrorResponse { error: e.to_string() });
    }

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
    rbac_service: web::Data<RbacService>,
    query_id: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let extensions = req.extensions();
    let claims = match extensions.get::<Claims>() {
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

    let query = match analytics_service.get_query_by_id(&query_uuid).await {
        Ok(q) => q,
        Err(e) => {
            return HttpResponse::NotFound().json(ErrorResponse { error: e });
        }
    };

    // Check permission to read reports in this project
    if let Err(e) = check_permission(
        &rbac_service,
        &claims.user_id,
        Some(&query.project_id),
        Permission::ReportRead
    ).await {
        return HttpResponse::Forbidden().json(ErrorResponse { error: e.to_string() });
    }

    HttpResponse::Ok().json(query)
}

pub async fn get_project_queries(
    analytics_service: web::Data<AnalyticsService>,
    rbac_service: web::Data<RbacService>,
    project_id: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let extensions = req.extensions();
    let claims = match extensions.get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Unauthorized".to_string(),
            });
        }
    };

    let project_id_str = project_id.into_inner();

    // Check permission to read reports in this project
    if let Err(e) = check_permission(
        &rbac_service,
        &claims.user_id,
        Some(&project_id_str),
        Permission::ReportRead
    ).await {
        return HttpResponse::Forbidden().json(ErrorResponse { error: e.to_string() });
    }

    let project_uuid = match uuid::Uuid::parse_str(&project_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid project ID".to_string(),
            });
        }
    };

    match analytics_service.get_project_queries(&project_uuid).await {
        Ok(queries) => HttpResponse::Ok().json(queries),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: e }),
    }
}
