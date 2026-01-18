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
    println!("Initializing AI service with provider: {:?}", config.ai_provider);
    let ai_service = services::AIService::new(
        config.ai_api_url.clone(),
        config.ai_model_name.clone(),
        config.ai_provider.clone(),
        config.ai_api_key.clone(),
    );

    // Initialize services
    let user_service = web::Data::new(services::UserService::new(db_manager.clone()));
    let project_service = web::Data::new(services::ProjectService::new(db_manager.clone()));
    let analytics_service = web::Data::new(services::AnalyticsService::new(
        db_manager.clone(),
        ai_service.clone(),
    ));
    let chat_service = web::Data::new(services::ChatService::new(
        db_manager.clone(),
        ai_service,
        config.chat_rate_limit_messages,
        config.chat_rate_limit_window_secs,
        config.chat_context_message_limit,
    ));
    let rbac_service = web::Data::new(services::RbacService::new(db_manager.clone()));

    // Ensure system roles exist
    rbac_service.ensure_system_roles()
        .await
        .expect("Failed to seed system roles");

    let jwt_manager_data = web::Data::new(jwt_manager.clone());
    let redis = db_manager.redis.clone();
    let cors_origins = config.cors_allowed_origins.clone();
    let rate_limit_requests = config.rate_limit_requests;
    let rate_limit_window_secs = config.rate_limit_window_secs;

    log::info!("All services initialized successfully");

    // Start HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors_origins_clone = cors_origins.clone();
        let cors = Cors::default()
            .allowed_origin_fn(move |origin, _req_head| {
                cors_origins_clone.iter().any(|allowed| {
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
            .app_data(chat_service.clone())
            .app_data(rbac_service.clone())
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
                        rate_limit_requests,
                        rate_limit_window_secs,
                    ))
                    .service(
                        web::scope("/users")
                            .route("/me", web::get().to(handlers::auth::get_current_user))
                            .route("/me/password", web::put().to(handlers::user::change_password))
                            .route("", web::get().to(handlers::user::get_users))
                            .route("", web::post().to(handlers::user::create_user))
                            .route("/search", web::get().to(handlers::user::search_users))
                            .route("/{user_id}", web::get().to(handlers::user::get_user))
                            .route("/{user_id}", web::put().to(handlers::user::update_user))
                            .route("/{user_id}", web::delete().to(handlers::user::delete_user))
                            .route("/{user_id}/password", web::put().to(handlers::user::reset_user_password))
                    )
                    .service(
                        web::scope("/projects")
                            .route("", web::post().to(handlers::project::create_project))
                            .route("", web::get().to(handlers::project::get_user_projects))
                            .route("/{project_id}", web::get().to(handlers::project::get_project_by_id))
                            .route("/{project_id}", web::put().to(handlers::project::update_project))
                            .route("/{project_id}", web::delete().to(handlers::project::delete_project))
                            .route("/{project_id}/members", web::get().to(handlers::rbac::get_project_members))
                    )
                    .service(
                        web::scope("/analytics")
                            .route("/queries", web::post().to(handlers::analytics::create_query))
                            .route("/queries/{query_id}", web::get().to(handlers::analytics::get_query_by_id))
                            .route("/queries/{query_id}/process", web::post().to(handlers::analytics::process_query))
                            .route("/projects/{project_id}/queries", web::get().to(handlers::analytics::get_project_queries))
                    )
                    .service(
                        web::scope("/chat")
                            .route("/message", web::post().to(handlers::chat::send_message))
                            .route("/conversations/{conversation_id}", web::get().to(handlers::chat::get_conversation))
                            .route("/projects/{project_id}/conversations", web::get().to(handlers::chat::get_project_conversations))
                            .route("/projects/{project_id}/conversations/summaries", web::get().to(handlers::chat::get_project_conversation_summaries))
                    )
                    .service(
                        web::scope("/rbac")
                            .route("/permissions", web::get().to(handlers::rbac::get_all_permissions))
                            .route("/permissions/me", web::get().to(handlers::rbac::get_my_permissions))
                            .route("/roles", web::post().to(handlers::rbac::create_role))
                            .route("/roles", web::get().to(handlers::rbac::get_roles))
                            .route("/roles/{role_id}", web::get().to(handlers::rbac::get_role))
                            .route("/roles/{role_id}", web::put().to(handlers::rbac::update_role))
                            .route("/roles/{role_id}", web::delete().to(handlers::rbac::delete_role))
                            .route("/memberships", web::post().to(handlers::rbac::assign_role))
                            .route("/memberships/me", web::get().to(handlers::rbac::get_my_memberships))
                            .route("/memberships/user/{user_id}", web::get().to(handlers::rbac::get_user_memberships))
                            .route("/memberships/{project_id}/{user_id}", web::delete().to(handlers::rbac::revoke_role))
                            .route("/initialize", web::post().to(handlers::rbac::initialize_system_roles))
                    )
            )
    })
    .bind(&server_addr)?
    .run()
    .await
}
