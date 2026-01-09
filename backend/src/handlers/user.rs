use actix_web::{web, HttpResponse, HttpMessage, HttpRequest};
use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::models::{AdminCreateUserDto, UpdateUserDto, ChangePasswordDto, ResetPasswordDto, UserResponse, Permission};
use crate::services::{UserService, RbacService};
use crate::middleware::rbac::check_permission;
use crate::utils::Claims;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Serialize)]
struct MessageResponse {
    message: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    #[serde(default)]
    pub permanent: bool,
}

/// Get all users in the tenant
pub async fn get_users(
    req: HttpRequest,
    user_service: web::Data<UserService>,
    rbac_service: web::Data<RbacService>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Not authenticated".to_string(),
            });
        }
    };

    // Check permission
    if let Err(_) = check_permission(&rbac_service, &claims.sub.to_string(), None, Permission::UserRead).await {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Insufficient permissions".to_string(),
        });
    }

    match user_service.get_users_by_tenant(&claims.tenant_id).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: e }),
    }
}

/// Search users
pub async fn search_users(
    req: HttpRequest,
    user_service: web::Data<UserService>,
    rbac_service: web::Data<RbacService>,
    query: web::Query<SearchQuery>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Not authenticated".to_string(),
            });
        }
    };

    // Check permission
    if let Err(_) = check_permission(&rbac_service, &claims.sub.to_string(), None, Permission::UserRead).await {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Insufficient permissions".to_string(),
        });
    }

    match user_service.search_users(&claims.tenant_id, &query.q).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: e }),
    }
}

/// Get a specific user by ID
pub async fn get_user(
    req: HttpRequest,
    user_service: web::Data<UserService>,
    rbac_service: web::Data<RbacService>,
    path: web::Path<String>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Not authenticated".to_string(),
            });
        }
    };

    let user_id = path.into_inner();
    
    // Users can always view themselves, otherwise need permission
    if user_id != claims.sub.to_string() {
        if let Err(_) = check_permission(&rbac_service, &claims.sub.to_string(), None, Permission::UserRead).await {
            return HttpResponse::Forbidden().json(ErrorResponse {
                error: "Insufficient permissions".to_string(),
            });
        }
    }

    match user_service.get_user_by_id_str(&user_id).await {
        Ok(user) => {
            // Verify tenant
            if user.tenant_id != claims.tenant_id {
                return HttpResponse::NotFound().json(ErrorResponse {
                    error: "User not found".to_string(),
                });
            }
            HttpResponse::Ok().json(UserResponse::from(user))
        }
        Err(e) => HttpResponse::NotFound().json(ErrorResponse { error: e }),
    }
}

/// Get current user (self)
pub async fn get_current_user(
    req: HttpRequest,
    user_service: web::Data<UserService>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Not authenticated".to_string(),
            });
        }
    };

    match user_service.get_user_by_id_str(&claims.sub.to_string()).await {
        Ok(user) => HttpResponse::Ok().json(UserResponse::from(user)),
        Err(e) => HttpResponse::NotFound().json(ErrorResponse { error: e }),
    }
}

/// Create a new user (admin only)
pub async fn create_user(
    req: HttpRequest,
    user_service: web::Data<UserService>,
    rbac_service: web::Data<RbacService>,
    dto: web::Json<AdminCreateUserDto>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Not authenticated".to_string(),
            });
        }
    };

    // Check permission
    if let Err(_) = check_permission(&rbac_service, &claims.sub.to_string(), None, Permission::UserCreate).await {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Insufficient permissions".to_string(),
        });
    }

    if let Err(e) = dto.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Validation error: {}", e),
        });
    }

    match user_service.admin_create_user(dto.into_inner(), &claims.tenant_id).await {
        Ok(user) => HttpResponse::Created().json(user),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse { error: e }),
    }
}

/// Update a user
pub async fn update_user(
    req: HttpRequest,
    user_service: web::Data<UserService>,
    rbac_service: web::Data<RbacService>,
    path: web::Path<String>,
    dto: web::Json<UpdateUserDto>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Not authenticated".to_string(),
            });
        }
    };

    let user_id = path.into_inner();
    let is_self = user_id == claims.sub.to_string();

    // Self can only update name, not role or is_active
    if is_self {
        if dto.role.is_some() || dto.is_active.is_some() {
            return HttpResponse::Forbidden().json(ErrorResponse {
                error: "Cannot change your own role or status".to_string(),
            });
        }
    } else {
        // Need permission to update others
        if let Err(_) = check_permission(&rbac_service, &claims.sub.to_string(), None, Permission::UserUpdate).await {
            return HttpResponse::Forbidden().json(ErrorResponse {
                error: "Insufficient permissions".to_string(),
            });
        }
    }

    // If changing role, need UserManageRoles permission
    if dto.role.is_some() {
        if let Err(_) = check_permission(&rbac_service, &claims.sub.to_string(), None, Permission::UserManageRoles).await {
            return HttpResponse::Forbidden().json(ErrorResponse {
                error: "Insufficient permissions to change roles".to_string(),
            });
        }
    }

    match user_service.update_user(&user_id, dto.into_inner(), &claims.tenant_id).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse { error: e }),
    }
}

/// Change own password
pub async fn change_password(
    req: HttpRequest,
    user_service: web::Data<UserService>,
    dto: web::Json<ChangePasswordDto>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Not authenticated".to_string(),
            });
        }
    };

    if let Err(e) = dto.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Validation error: {}", e),
        });
    }

    match user_service.change_password(&claims.sub.to_string(), &dto.current_password, &dto.new_password).await {
        Ok(_) => HttpResponse::Ok().json(MessageResponse {
            message: "Password changed successfully".to_string(),
        }),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse { error: e }),
    }
}

/// Reset a user's password (admin only)
pub async fn reset_user_password(
    req: HttpRequest,
    user_service: web::Data<UserService>,
    rbac_service: web::Data<RbacService>,
    path: web::Path<String>,
    dto: web::Json<ResetPasswordDto>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Not authenticated".to_string(),
            });
        }
    };

    // Check permission
    if let Err(_) = check_permission(&rbac_service, &claims.sub.to_string(), None, Permission::UserUpdate).await {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Insufficient permissions".to_string(),
        });
    }

    let user_id = path.into_inner();

    if let Err(e) = dto.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Validation error: {}", e),
        });
    }

    match user_service.reset_password(&user_id, &claims.tenant_id, &dto.new_password).await {
        Ok(_) => HttpResponse::Ok().json(MessageResponse {
            message: "Password reset successfully".to_string(),
        }),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse { error: e }),
    }
}

/// Delete a user
pub async fn delete_user(
    req: HttpRequest,
    user_service: web::Data<UserService>,
    rbac_service: web::Data<RbacService>,
    path: web::Path<String>,
    query: web::Query<DeleteQuery>,
) -> HttpResponse {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Not authenticated".to_string(),
            });
        }
    };

    // Check permission
    if let Err(_) = check_permission(&rbac_service, &claims.sub.to_string(), None, Permission::UserDelete).await {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Insufficient permissions".to_string(),
        });
    }

    let user_id = path.into_inner();

    // Cannot delete self
    if user_id == claims.sub.to_string() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Cannot delete your own account".to_string(),
        });
    }

    match user_service.delete_user(&user_id, &claims.tenant_id, query.permanent).await {
        Ok(_) => HttpResponse::Ok().json(MessageResponse {
            message: if query.permanent {
                "User deleted permanently".to_string()
            } else {
                "User deactivated successfully".to_string()
            },
        }),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse { error: e }),
    }
}
