use actix_web::{web, HttpResponse, HttpRequest};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;
use crate::models::{CreateProjectDto, ProjectResponse};
use crate::services::ProjectService;
use crate::utils::Claims;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn create_project(
    project_service: web::Data<ProjectService>,
    dto: web::Json<CreateProjectDto>,
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

    match project_service
        .create_project(dto.into_inner(), &user_id, &claims.tenant_id)
        .await
    {
        Ok(project) => HttpResponse::Created().json(project),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse { error: e }),
    }
}

pub async fn get_user_projects(
    project_service: web::Data<ProjectService>,
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

    let user_id = match uuid::Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user ID".to_string(),
            });
        }
    };

    match project_service
        .get_user_projects(&user_id, &claims.tenant_id)
        .await
    {
        Ok(projects) => HttpResponse::Ok().json(projects),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: e }),
    }
}

pub async fn get_project_by_id(
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

    // Check access
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

    match project_service.get_project_by_id(&project_uuid).await {
        Ok(project) => HttpResponse::Ok().json(ProjectResponse::from(project)),
        Err(e) => HttpResponse::NotFound().json(ErrorResponse { error: e }),
    }
}
