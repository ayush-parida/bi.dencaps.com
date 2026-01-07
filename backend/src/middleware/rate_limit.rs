use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::future::LocalBoxFuture;
use redis::AsyncCommands;
use std::future::{ready, Ready};
use std::sync::Arc;
use redis::aio::ConnectionManager;

pub struct RateLimitMiddleware {
    redis: Arc<ConnectionManager>,
    max_requests: usize,
    window_secs: u64,
}

impl RateLimitMiddleware {
    pub fn new(redis: Arc<ConnectionManager>, max_requests: usize, window_secs: u64) -> Self {
        RateLimitMiddleware {
            redis,
            max_requests,
            window_secs,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service,
            redis: self.redis.clone(),
            max_requests: self.max_requests,
            window_secs: self.window_secs,
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: S,
    redis: Arc<ConnectionManager>,
    max_requests: usize,
    window_secs: u64,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
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
        let redis = self.redis.clone();
        let max_requests = self.max_requests;
        let window_secs = self.window_secs;

        // Get client IP
        let client_ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        let key = format!("rate_limit:{}", client_ip);
        
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut conn = redis.as_ref().clone();
            
            // Increment counter
            let count: Result<i32, redis::RedisError> = conn.incr(&key, 1).await;
            
            match count {
                Ok(cnt) => {
                    if cnt == 1 {
                        // Set expiration on first request
                        let _: Result<(), redis::RedisError> = conn.expire(&key, window_secs as i64).await;
                    }

                    if cnt as usize > max_requests {
                        return Err(actix_web::error::ErrorTooManyRequests(
                            "Rate limit exceeded"
                        ));
                    }

                    fut.await
                }
                Err(_) => {
                    // If Redis fails, allow the request through
                    fut.await
                }
            }
        })
    }
}
