use actix_web::{web, HttpRequest, HttpResponse, HttpMessage};
use crate::models::{
    Permission, CreateRoleDto, UpdateRoleDto, AssignRoleDto,
    RoleResponse, ProjectMembershipResponse, UserPermissionsResponse,
};
use crate::services::RbacService;
use crate::utils::Claims;
use crate::middleware::check_permission;
use validator::Validate;

/// Get all available permissions in the system
pub async fn get_all_permissions() -> HttpResponse {
    let permissions: Vec<&str> = Permission::all()
        .iter()
        .map(|p| p.as_str())
        .collect();
    
    HttpResponse::Ok().json(permissions)
}

/// Get current user's permissions for a project
pub async fn get_my_permissions(
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
    query: web::Query<OptionalProjectQuery>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Not authenticated"})),
    };

    let project_id = query.project_id.as_deref();

    match rbac_service.resolve_permissions(&claims.user_id, project_id).await {
        Ok(resolved) => {
            let response: UserPermissionsResponse = resolved.into();
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("Failed to get permissions: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
        }
    }
}

/// Create a new role (requires admin or user:manage_roles permission)
pub async fn create_role(
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
    body: web::Json<CreateRoleDto>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Not authenticated"})),
    };

    // Check permission
    if let Err(e) = check_permission(&rbac_service, &claims.user_id, None, Permission::UserManageRoles).await {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": e.to_string()}));
    }

    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()}));
    }

    match rbac_service.create_role(body.into_inner(), &claims.tenant_id).await {
        Ok(role) => {
            let response: RoleResponse = role.into();
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            log::error!("Failed to create role: {}", e);
            HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
        }
    }
}

/// Get all roles for the tenant
pub async fn get_roles(
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Not authenticated"})),
    };

    // Check permission
    if let Err(e) = check_permission(&rbac_service, &claims.user_id, None, Permission::UserRead).await {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": e.to_string()}));
    }

    match rbac_service.get_tenant_roles(&claims.tenant_id).await {
        Ok(roles) => {
            let responses: Vec<RoleResponse> = roles.into_iter().map(|r| r.into()).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => {
            log::error!("Failed to get roles: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
        }
    }
}

/// Get a specific role by ID
pub async fn get_role(
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Not authenticated"})),
    };

    let role_id = path.into_inner();

    // Check permission
    if let Err(e) = check_permission(&rbac_service, &claims.user_id, None, Permission::UserRead).await {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": e.to_string()}));
    }

    match rbac_service.get_role_by_id(&role_id).await {
        Ok(Some(role)) => {
            if role.tenant_id != claims.tenant_id {
                return HttpResponse::NotFound().json(serde_json::json!({"error": "Role not found"}));
            }
            let response: RoleResponse = role.into();
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "Role not found"})),
        Err(e) => {
            log::error!("Failed to get role: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
        }
    }
}

/// Update a role
pub async fn update_role(
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<UpdateRoleDto>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Not authenticated"})),
    };

    let role_id = path.into_inner();

    // Check permission
    if let Err(e) = check_permission(&rbac_service, &claims.user_id, None, Permission::UserManageRoles).await {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": e.to_string()}));
    }

    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()}));
    }

    match rbac_service.update_role(&role_id, body.into_inner(), &claims.tenant_id).await {
        Ok(role) => {
            let response: RoleResponse = role.into();
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("Failed to update role: {}", e);
            HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
        }
    }
}

/// Delete a role
pub async fn delete_role(
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Not authenticated"})),
    };

    let role_id = path.into_inner();

    // Check permission
    if let Err(e) = check_permission(&rbac_service, &claims.user_id, None, Permission::UserManageRoles).await {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": e.to_string()}));
    }

    match rbac_service.delete_role(&role_id, &claims.tenant_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => {
            log::error!("Failed to delete role: {}", e);
            HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
        }
    }
}

/// Assign a role to a user for a project
pub async fn assign_role(
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
    body: web::Json<AssignRoleDto>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Not authenticated"})),
    };

    // Check permission - need project manage members or admin
    if let Err(e) = check_permission(
        &rbac_service, 
        &claims.user_id, 
        Some(&body.project_id), 
        Permission::ProjectManageMembers
    ).await {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": e.to_string()}));
    }

    if let Err(e) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()}));
    }

    match rbac_service.assign_role(body.into_inner(), &claims.tenant_id).await {
        Ok(membership) => {
            let response = ProjectMembershipResponse {
                membership_id: membership.membership_id,
                user_id: membership.user_id,
                project_id: membership.project_id,
                role_id: membership.role_id,
                role_name: None,
                created_at: membership.created_at.to_string(),
            };
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            log::error!("Failed to assign role: {}", e);
            HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
        }
    }
}

/// Revoke a role from a user for a project
pub async fn revoke_role(
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Not authenticated"})),
    };

    let (project_id, user_id) = path.into_inner();

    // Check permission
    if let Err(e) = check_permission(
        &rbac_service, 
        &claims.user_id, 
        Some(&project_id), 
        Permission::ProjectManageMembers
    ).await {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": e.to_string()}));
    }

    match rbac_service.revoke_role(&user_id, &project_id, &claims.tenant_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => {
            log::error!("Failed to revoke role: {}", e);
            HttpResponse::BadRequest().json(serde_json::json!({"error": e}))
        }
    }
}

/// Get all members of a project
pub async fn get_project_members(
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Not authenticated"})),
    };

    let project_id = path.into_inner();

    // Check permission - need at least project read
    if let Err(e) = check_permission(
        &rbac_service, 
        &claims.user_id, 
        Some(&project_id), 
        Permission::ProjectRead
    ).await {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": e.to_string()}));
    }

    match rbac_service.get_project_memberships(&project_id).await {
        Ok(memberships) => {
            let mut responses = Vec::new();
            for m in memberships {
                let role_name = if let Ok(Some(role)) = rbac_service.get_role_by_id(&m.role_id).await {
                    Some(role.name)
                } else {
                    None
                };
                responses.push(ProjectMembershipResponse {
                    membership_id: m.membership_id,
                    user_id: m.user_id,
                    project_id: m.project_id,
                    role_id: m.role_id,
                    role_name,
                    created_at: m.created_at.to_string(),
                });
            }
            HttpResponse::Ok().json(responses)
        }
        Err(e) => {
            log::error!("Failed to get project members: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
        }
    }
}

/// Get user's memberships across all projects
pub async fn get_my_memberships(
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Not authenticated"})),
    };

    match rbac_service.get_user_memberships(&claims.user_id).await {
        Ok(memberships) => {
            let mut responses = Vec::new();
            for m in memberships {
                let role_name = if let Ok(Some(role)) = rbac_service.get_role_by_id(&m.role_id).await {
                    Some(role.name)
                } else {
                    None
                };
                responses.push(ProjectMembershipResponse {
                    membership_id: m.membership_id,
                    user_id: m.user_id,
                    project_id: m.project_id,
                    role_id: m.role_id,
                    role_name,
                    created_at: m.created_at.to_string(),
                });
            }
            HttpResponse::Ok().json(responses)
        }
        Err(e) => {
            log::error!("Failed to get memberships: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
        }
    }
}

/// Initialize system roles for the tenant (admin only)
pub async fn initialize_system_roles(
    rbac_service: web::Data<RbacService>,
    req: HttpRequest,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Not authenticated"})),
    };

    // Check permission - admin only
    if let Err(e) = check_permission(&rbac_service, &claims.user_id, None, Permission::AdminAccess).await {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": e.to_string()}));
    }

    match rbac_service.initialize_system_roles(&claims.tenant_id).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"message": "System roles initialized"})),
        Err(e) => {
            log::error!("Failed to initialize system roles: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
        }
    }
}

#[derive(serde::Deserialize)]
pub struct OptionalProjectQuery {
    pub project_id: Option<String>,
}
