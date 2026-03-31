use actix_web::{web, App, HttpServer, HttpResponse, middleware::Logger};
use tracing_subscriber::EnvFilter;

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "Friendship&Service"
    }))
}

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body("<h1>Friendship&amp;Service API</h1><p>Backend is running.</p>")
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

    tracing::info!("Starting Friendship&Service on {}:{}", host, port);

    HttpServer::new(|| {
        let cors = actix_cors::Cors::permissive();

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .route("/", web::get().to(index))
            .route("/api/health", web::get().to(health))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
