use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use serde::Serialize;
use validator::Validate;
use crate::models::{CreateProjectDto, ProjectResponse, Permission};
use crate::services::{ProjectService, RbacService};
use crate::utils::Claims;
use crate::middleware::check_permission;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn create_project(
    project_service: web::Data<ProjectService>,
    rbac_service: web::Data<RbacService>,
    dto: web::Json<CreateProjectDto>,
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

    // Check permission to create projects
    if let Err(e) = check_permission(&rbac_service, &claims.user_id, None, Permission::ProjectCreate).await {
        return HttpResponse::Forbidden().json(ErrorResponse { error: e.to_string() });
    }

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
    rbac_service: web::Data<RbacService>,
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

    let user_id = match uuid::Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user ID".to_string(),
            });
        }
    };

    // Resolve permissions to check if admin
    let permissions = match rbac_service.resolve_permissions(&claims.user_id, None).await {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to resolve permissions: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse { error: e });
        }
    };

    // Admin users can see all projects in their tenant
    log::info!("User role: '{}', is_admin: {}", claims.role, permissions.is_admin);
    let result = if permissions.is_admin {
        log::info!("Admin user detected, fetching all tenant projects for tenant_id: {}", claims.tenant_id);
        project_service
            .get_projects_by_tenant(&claims.tenant_id)
            .await
    } else {
        // Get projects where user has membership or is owner
        project_service
            .get_user_projects(&user_id, &claims.tenant_id)
            .await
    };

    match result {
        Ok(ref projects) => log::info!("Found {} projects", projects.len()),
        Err(ref e) => log::error!("Error fetching projects: {}", e),
    }

    match result {
        Ok(projects) => HttpResponse::Ok().json(projects),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: e }),
    }
}

pub async fn get_project_by_id(
    project_service: web::Data<ProjectService>,
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

    // Check permission using RBAC - this verifies project access
    if let Err(e) = check_permission(
        &rbac_service, 
        &claims.user_id, 
        Some(&project_id_str), 
        Permission::ProjectRead
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

    match project_service.get_project_by_id(&project_uuid).await {
        Ok(project) => {
            // Verify tenant isolation
            if project.tenant_id != claims.tenant_id {
                return HttpResponse::Forbidden().json(ErrorResponse {
                    error: "Access denied".to_string(),
                });
            }
            HttpResponse::Ok().json(ProjectResponse::from(project))
        }
        Err(e) => HttpResponse::NotFound().json(ErrorResponse { error: e }),
    }
}

pub async fn update_project(
    project_service: web::Data<ProjectService>,
    rbac_service: web::Data<RbacService>,
    project_id: web::Path<String>,
    dto: web::Json<crate::models::UpdateProjectDto>,
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

    let project_id_str = project_id.into_inner();

    // Check permission to update project
    if let Err(e) = check_permission(
        &rbac_service,
        &claims.user_id,
        Some(&project_id_str),
        Permission::ProjectUpdate
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

    match project_service.update_project(&project_uuid, dto.into_inner(), &claims.tenant_id).await {
        Ok(project) => HttpResponse::Ok().json(ProjectResponse::from(project)),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse { error: e }),
    }
}

pub async fn delete_project(
    project_service: web::Data<ProjectService>,
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

    // Check permission to delete project
    if let Err(e) = check_permission(
        &rbac_service,
        &claims.user_id,
        Some(&project_id_str),
        Permission::ProjectDelete
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

    match project_service.delete_project(&project_uuid, &claims.tenant_id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse { error: e }),
    }
}
