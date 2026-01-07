mod config;
mod db;
mod models;
mod utils;
mod middleware;
mod services;
mod handlers;

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Starting DencapsBI Backend Server...");

    // Load configuration
    let config = config::Config::from_env()
        .expect("Failed to load configuration");

    let server_addr = format!("{}:{}", config.server_host, config.server_port);
    log::info!("Server will bind to: {}", server_addr);

    // Initialize database connections
    let db_manager = db::DatabaseManager::new(&config)
        .await
        .expect("Failed to initialize database connections");

    // Create indexes
    db_manager
        .create_indexes()
        .await
        .expect("Failed to create database indexes");

    // Initialize JWT manager
    let jwt_manager = Arc::new(utils::JwtManager::new(
        config.jwt_secret.clone(),
        config.jwt_expiration,
        config.jwt_refresh_expiration,
    ));

    // Initialize AI service
    let ai_service = services::AIService::new(
        config.lm_studio_api_url.clone(),
        config.lm_studio_model_name.clone(),
    );

    // Initialize services
    let user_service = web::Data::new(services::UserService::new(db_manager.clone()));
    let project_service = web::Data::new(services::ProjectService::new(db_manager.clone()));
    let analytics_service = web::Data::new(services::AnalyticsService::new(
        db_manager.clone(),
        ai_service,
    ));

    let jwt_manager_data = web::Data::new(jwt_manager.clone());
    let redis = db_manager.redis.clone();

    log::info!("All services initialized successfully");

    // Start HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req_head| {
                config.cors_allowed_origins.iter().any(|allowed| {
                    origin.as_bytes() == allowed.as_bytes()
                })
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(user_service.clone())
            .app_data(project_service.clone())
            .app_data(analytics_service.clone())
            .app_data(jwt_manager_data.clone())
            // Public routes
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(handlers::auth::register))
                    .route("/login", web::post().to(handlers::auth::login))
                    .route("/refresh", web::post().to(handlers::auth::refresh_token))
            )
            // Protected routes
            .service(
                web::scope("/api")
                    .wrap(middleware::AuthMiddleware::new(jwt_manager.clone()))
                    .wrap(middleware::RateLimitMiddleware::new(
                        redis.clone(),
                        config.rate_limit_requests,
                        config.rate_limit_window_secs,
                    ))
                    .service(
                        web::scope("/users")
                            .route("/me", web::get().to(handlers::auth::get_current_user))
                    )
                    .service(
                        web::scope("/projects")
                            .route("", web::post().to(handlers::project::create_project))
                            .route("", web::get().to(handlers::project::get_user_projects))
                            .route("/{project_id}", web::get().to(handlers::project::get_project_by_id))
                    )
                    .service(
                        web::scope("/analytics")
                            .route("/queries", web::post().to(handlers::analytics::create_query))
                            .route("/queries/{query_id}", web::get().to(handlers::analytics::get_query_by_id))
                            .route("/queries/{query_id}/process", web::post().to(handlers::analytics::process_query))
                            .route("/projects/{project_id}/queries", web::get().to(handlers::analytics::get_project_queries))
                    )
            )
    })
    .bind(&server_addr)?
    .run()
    .await
}
