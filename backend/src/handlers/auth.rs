use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;
use crate::models::{CreateUserDto, LoginDto, AuthResponse, UserResponse};
use crate::services::UserService;
use crate::utils::{JwtManager, Claims};

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn register(
    user_service: web::Data<UserService>,
    jwt_manager: web::Data<Arc<JwtManager>>,
    dto: web::Json<CreateUserDto>,
) -> HttpResponse {
    if let Err(e) = dto.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Validation error: {}", e),
        });
    }

    match user_service.create_user(dto.into_inner()).await {
        Ok(user) => {
            let access_token = match jwt_manager.generate_token(
                &uuid::Uuid::parse_str(&user.user_id).unwrap(),
                &user.email,
                &user.role,
                &user.tenant_id,
                false,
            ) {
                Ok(token) => token,
                Err(e) => {
                    return HttpResponse::InternalServerError().json(ErrorResponse {
                        error: format!("Failed to generate token: {}", e),
                    });
                }
            };

            let refresh_token = match jwt_manager.generate_token(
                &uuid::Uuid::parse_str(&user.user_id).unwrap(),
                &user.email,
                &user.role,
                &user.tenant_id,
                true,
            ) {
                Ok(token) => token,
                Err(e) => {
                    return HttpResponse::InternalServerError().json(ErrorResponse {
                        error: format!("Failed to generate refresh token: {}", e),
                    });
                }
            };

            HttpResponse::Created().json(AuthResponse {
                access_token,
                refresh_token,
                user,
            })
        }
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse { error: e }),
    }
}

pub async fn login(
    user_service: web::Data<UserService>,
    jwt_manager: web::Data<Arc<JwtManager>>,
    dto: web::Json<LoginDto>,
) -> HttpResponse {
    if let Err(e) = dto.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Validation error: {}", e),
        });
    }

    match user_service.authenticate(&dto.email, &dto.password).await {
        Ok(user) => {
            let access_token = match jwt_manager.generate_token(
                &user.user_id,
                &user.email,
                &user.role,
                &user.tenant_id,
                false,
            ) {
                Ok(token) => token,
                Err(e) => {
                    return HttpResponse::InternalServerError().json(ErrorResponse {
                        error: format!("Failed to generate token: {}", e),
                    });
                }
            };

            let refresh_token = match jwt_manager.generate_token(
                &user.user_id,
                &user.email,
                &user.role,
                &user.tenant_id,
                true,
            ) {
                Ok(token) => token,
                Err(e) => {
                    return HttpResponse::InternalServerError().json(ErrorResponse {
                        error: format!("Failed to generate refresh token: {}", e),
                    });
                }
            };

            HttpResponse::Ok().json(AuthResponse {
                access_token,
                refresh_token,
                user: UserResponse::from(user),
            })
        }
        Err(e) => HttpResponse::Unauthorized().json(ErrorResponse { error: e }),
    }
}

pub async fn get_current_user(
    user_service: web::Data<UserService>,
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

    match user_service.get_user_by_id(&user_id).await {
        Ok(user) => HttpResponse::Ok().json(UserResponse::from(user)),
        Err(e) => HttpResponse::NotFound().json(ErrorResponse { error: e }),
    }
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    refresh_token: String,
}

pub async fn refresh_token(
    jwt_manager: web::Data<Arc<JwtManager>>,
    dto: web::Json<RefreshTokenRequest>,
) -> HttpResponse {
    match jwt_manager.validate_token(&dto.refresh_token) {
        Ok(claims) => {
            let user_id = match uuid::Uuid::parse_str(&claims.user_id) {
                Ok(id) => id,
                Err(_) => {
                    return HttpResponse::BadRequest().json(ErrorResponse {
                        error: "Invalid user ID".to_string(),
                    });
                }
            };

            let access_token = match jwt_manager.generate_token(
                &user_id,
                &claims.email,
                &claims.role,
                &claims.tenant_id,
                false,
            ) {
                Ok(token) => token,
                Err(e) => {
                    return HttpResponse::InternalServerError().json(ErrorResponse {
                        error: format!("Failed to generate token: {}", e),
                    });
                }
            };

            HttpResponse::Ok().json(serde_json::json!({
                "access_token": access_token,
            }))
        }
        Err(e) => HttpResponse::Unauthorized().json(ErrorResponse { error: e }),
    }
}
