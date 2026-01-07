use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::sync::Arc;
use crate::utils::JwtManager;

pub struct AuthMiddleware {
    jwt_manager: Arc<JwtManager>,
}

impl AuthMiddleware {
    pub fn new(jwt_manager: Arc<JwtManager>) -> Self {
        AuthMiddleware { jwt_manager }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service,
            jwt_manager: self.jwt_manager.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    jwt_manager: Arc<JwtManager>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let jwt_manager = self.jwt_manager.clone();
        
        // Extract token from Authorization header
        let auth_header = req.headers().get("Authorization");
        
        let token = if let Some(auth_value) = auth_header {
            if let Ok(auth_str) = auth_value.to_str() {
                if auth_str.starts_with("Bearer ") {
                    Some(auth_str[7..].to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(token_str) = token {
            match jwt_manager.validate_token(&token_str) {
                Ok(claims) => {
                    req.extensions_mut().insert(claims);
                    let fut = self.service.call(req);
                    Box::pin(async move {
                        let res = fut.await?;
                        Ok(res)
                    })
                }
                Err(_) => {
                    Box::pin(async move {
                        Err(actix_web::error::ErrorUnauthorized("Invalid token"))
                    })
                }
            }
        } else {
            Box::pin(async move {
                Err(actix_web::error::ErrorUnauthorized("Missing authorization token"))
            })
        }
    }
}
