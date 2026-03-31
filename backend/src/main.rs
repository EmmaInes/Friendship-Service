use actix_web::{web, App, HttpServer, HttpResponse, middleware::Logger};
use rusqlite::Connection;
use std::sync::Mutex;
use tracing_subscriber::EnvFilter;

mod auth;
mod db;
mod handlers;
mod models;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub jwt_secret: String,
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "Friendship&Service"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let host = std::env::var("FS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = std::env::var("FS_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("FS_PORT must be a valid port number");

    let jwt_secret = std::env::var("FS_JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret-change-in-production".to_string());

    let db_path = std::env::var("FS_DB_PATH").unwrap_or_else(|_| "data/app.db".to_string());
    let conn = db::init(&db_path);

    let state = web::Data::new(AppState {
        db: Mutex::new(conn),
        jwt_secret,
    });

    tracing::info!("Starting Friendship&Service on {}:{}", host, port);

    HttpServer::new(move || {
        let cors = actix_cors::Cors::permissive();

        App::new()
            .app_data(state.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .route("/api/health", web::get().to(health))
            .route("/api/auth/register", web::post().to(handlers::auth::register))
            .route("/api/auth/login", web::post().to(handlers::auth::login))
            .route("/api/auth/me", web::get().to(handlers::auth::me))
            // Services
            .route("/api/services", web::get().to(handlers::services::list))
            .route("/api/services", web::post().to(handlers::services::create))
            .route("/api/services/mine", web::get().to(handlers::services::mine))
            .route("/api/services/{id}", web::get().to(handlers::services::get))
            .route("/api/services/{id}/request", web::post().to(handlers::services::request_service))
            // Requests
            .route("/api/requests/mine", web::get().to(handlers::services::my_requests))
            .route("/api/requests/{id}", web::patch().to(handlers::services::update_request_status))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
