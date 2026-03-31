use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::auth;
use crate::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3, max = 30))]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn register(
    state: web::Data<AppState>,
    body: web::Json<RegisterRequest>,
) -> HttpResponse {
    if let Err(errors) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    let password_hash = match auth::hash_password(&body.password) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to hash password"
            }))
        }
    };

    let id = Uuid::new_v4().to_string();
    let db = state.db.lock().unwrap();

    let result = db.execute(
        "INSERT INTO users (id, email, username, password_hash, display_name) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, body.email, body.username, password_hash, body.display_name],
    );

    match result {
        Ok(_) => {
            let token = auth::create_token(&id, &state.jwt_secret).unwrap();
            HttpResponse::Created().json(serde_json::json!({
                "token": token,
                "user": {
                    "id": id,
                    "email": body.email,
                    "username": body.username,
                    "display_name": body.display_name,
                    "role": "both"
                }
            }))
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("UNIQUE") {
                HttpResponse::Conflict().json(serde_json::json!({
                    "error": "Email or username already taken"
                }))
            } else {
                tracing::error!("Registration failed: {}", msg);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Registration failed"
                }))
            }
        }
    }
}

pub async fn login(
    state: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> HttpResponse {
    let db = state.db.lock().unwrap();

    let result = db.query_row(
        "SELECT id, password_hash, email, username, display_name, role FROM users WHERE email = ?1",
        rusqlite::params![body.email],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        },
    );

    match result {
        Ok((id, hash, email, username, display_name, role)) => {
            if auth::verify_password(&body.password, &hash) {
                let token = auth::create_token(&id, &state.jwt_secret).unwrap();
                HttpResponse::Ok().json(serde_json::json!({
                    "token": token,
                    "user": {
                        "id": id,
                        "email": email,
                        "username": username,
                        "display_name": display_name,
                        "role": role
                    }
                }))
            } else {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Invalid credentials"
                }))
            }
        }
        Err(_) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid credentials"
        })),
    }
}

pub async fn me(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let token = match req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        Some(t) => t,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Missing or invalid Authorization header"
            }))
        }
    };

    let claims = match auth::validate_token(token, &state.jwt_secret) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or expired token"
            }))
        }
    };

    let db = state.db.lock().unwrap();
    let result = db.query_row(
        "SELECT id, email, username, display_name, bio, role, created_at FROM users WHERE id = ?1",
        rusqlite::params![claims.sub],
        |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "email": row.get::<_, String>(1)?,
                "username": row.get::<_, String>(2)?,
                "display_name": row.get::<_, String>(3)?,
                "bio": row.get::<_, String>(4)?,
                "role": row.get::<_, String>(5)?,
                "created_at": row.get::<_, String>(6)?,
            }))
        },
    );

    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "User not found"
        })),
    }
}
