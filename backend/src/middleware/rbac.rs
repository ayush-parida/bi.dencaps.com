use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage,
};
use futures::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::sync::Arc;
use crate::models::{Permission, ResolvedPermissions};
use crate::services::RbacService;
use crate::utils::Claims;

/// Middleware configuration for permission requirements
#[derive(Clone)]
pub struct RequirePermission {
    pub permissions: Vec<Permission>,
    pub require_all: bool,
    pub extract_project_from_path: bool,
}

impl RequirePermission {
    /// Require a single permission
    pub fn single(permission: Permission) -> Self {
        RequirePermission {
            permissions: vec![permission],
            require_all: true,
            extract_project_from_path: false,
        }
    }

    /// Require any of the given permissions
    pub fn any_of(permissions: Vec<Permission>) -> Self {
        RequirePermission {
            permissions,
            require_all: false,
            extract_project_from_path: false,
        }
    }

    /// Require all of the given permissions
    pub fn all_of(permissions: Vec<Permission>) -> Self {
        RequirePermission {
            permissions,
            require_all: true,
            extract_project_from_path: false,
        }
    }

    /// Extract project_id from path parameter for permission resolution
    pub fn with_project_from_path(mut self) -> Self {
        self.extract_project_from_path = true;
        self
    }
}

/// RBAC Middleware that enforces permissions on routes
pub struct RbacMiddleware {
    rbac_service: Arc<RbacService>,
    config: RequirePermission,
}

impl RbacMiddleware {
    pub fn new(rbac_service: Arc<RbacService>, config: RequirePermission) -> Self {
        RbacMiddleware { rbac_service, config }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RbacMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RbacMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RbacMiddlewareService {
            service: Arc::new(service),
            rbac_service: self.rbac_service.clone(),
            config: self.config.clone(),
        }))
    }
}

pub struct RbacMiddlewareService<S> {
    service: Arc<S>,
    rbac_service: Arc<RbacService>,
    config: RequirePermission,
}

impl<S, B> Service<ServiceRequest> for RbacMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let rbac_service = self.rbac_service.clone();
        let config = self.config.clone();

        Box::pin(async move {
            // Get claims from request extensions (set by auth middleware)
            let claims = req.extensions().get::<Claims>().cloned();
            
            let claims = match claims {
                Some(c) => c,
                None => {
                    return Err(actix_web::error::ErrorUnauthorized(
                        "Authentication required"
                    ));
                }
            };

            // Extract project_id from path if configured
            let project_id = if config.extract_project_from_path {
                req.match_info().get("project_id").map(|s| s.to_string())
            } else {
                None
            };

            // Resolve permissions
            let resolved = rbac_service
                .resolve_permissions(&claims.user_id, project_id.as_deref())
                .await
                .map_err(|e| {
                    log::error!("Failed to resolve permissions: {}", e);
                    actix_web::error::ErrorInternalServerError("Permission resolution failed")
                })?;

            // Check permissions
            let has_permission = if config.require_all {
                resolved.has_all_permissions(&config.permissions)
            } else {
                resolved.has_any_permission(&config.permissions)
            };

            if !has_permission {
                log::warn!(
                    "Permission denied for user {} on {:?}",
                    claims.user_id,
                    config.permissions.iter().map(|p| p.as_str()).collect::<Vec<_>>()
                );
                return Err(actix_web::error::ErrorForbidden(
                    "Insufficient permissions"
                ));
            }

            // Store resolved permissions in request extensions for handlers
            req.extensions_mut().insert(resolved);

            // Continue to the handler
            let fut = service.call(req);
            fut.await
        })
    }
}

/// Helper function to check permissions inline in handlers
pub async fn check_permission(
    rbac_service: &web::Data<RbacService>,
    user_id: &str,
    project_id: Option<&str>,
    permission: Permission,
) -> Result<ResolvedPermissions, actix_web::Error> {
    let resolved = rbac_service
        .resolve_permissions(user_id, project_id)
        .await
        .map_err(|e| {
            log::error!("Permission check failed: {}", e);
            actix_web::error::ErrorInternalServerError("Permission check failed")
        })?;

    if !resolved.has_permission(permission) {
        log::warn!(
            "Permission {} denied for user {} on project {:?}",
            permission.as_str(),
            user_id,
            project_id
        );
        return Err(actix_web::error::ErrorForbidden("Insufficient permissions"));
    }

    Ok(resolved)
}

/// Check if user has access to a specific project (prevents cross-project access)
pub async fn verify_project_access(
    rbac_service: &web::Data<RbacService>,
    user_id: &str,
    project_id: &str,
    tenant_id: &str,
) -> Result<ResolvedPermissions, actix_web::Error> {
    let resolved = rbac_service
        .resolve_permissions(user_id, Some(project_id))
        .await
        .map_err(|e| {
            log::error!("Project access check failed: {}", e);
            actix_web::error::ErrorInternalServerError("Access check failed")
        })?;

    // Admins bypass project access checks
    if resolved.is_admin {
        return Ok(resolved);
    }

    // Must have at least read permission on the project
    if !resolved.has_permission(Permission::ProjectRead) {
        log::warn!(
            "Project access denied for user {} on project {}",
            user_id,
            project_id
        );
        return Err(actix_web::error::ErrorForbidden(
            "You do not have access to this project"
        ));
    }

    Ok(resolved)
}

/// Extract resolved permissions from request (set by RBAC middleware)
pub fn get_resolved_permissions(req: &actix_web::HttpRequest) -> Option<ResolvedPermissions> {
    req.extensions().get::<ResolvedPermissions>().cloned()
}
