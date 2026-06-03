use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::auth;
use crate::AppState;
use super::extract_user_id;

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
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to hash password"
        })),
    };

    let id = Uuid::new_v4().to_string();

    let result = sqlx::query!(
        "INSERT INTO users (id, email, username, password_hash, display_name) VALUES ($1, $2, $3, $4, $5)",
        id, body.email, body.username, password_hash, body.display_name
    )
    .execute(&state.db)
    .await;

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
        Err(sqlx::Error::Database(e)) if e.constraint().is_some() => {
            HttpResponse::Conflict().json(serde_json::json!({
                "error": "Email or username already taken"
            }))
        }
        Err(e) => {
            tracing::error!("Registration failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Registration failed"
            }))
        }
    }
}

pub async fn login(
    state: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> HttpResponse {
    let result = sqlx::query!(
        "SELECT id, password_hash, email, username, display_name, role FROM users WHERE email = $1",
        body.email
    )
    .fetch_optional(&state.db)
    .await;

    match result {
        Ok(Some(row)) => {
            let hash = match row.password_hash {
                Some(h) => h,
                None => return HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "This account uses Google Sign-In"
                })),
            };

            if auth::verify_password(&body.password, &hash) {
                let token = auth::create_token(&row.id, &state.jwt_secret).unwrap();
                HttpResponse::Ok().json(serde_json::json!({
                    "token": token,
                    "user": {
                        "id": row.id,
                        "email": row.email,
                        "username": row.username,
                        "display_name": row.display_name,
                        "role": row.role
                    }
                }))
            } else {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Invalid credentials"
                }))
            }
        }
        _ => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid credentials"
        })),
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3, max = 30))]
    pub username: String,
    #[validate(length(min = 8))]
    pub new_password: String,
}

pub async fn reset_password(
    state: web::Data<AppState>,
    body: web::Json<ResetPasswordRequest>,
) -> HttpResponse {
    if let Err(errors) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    let user = sqlx::query!(
        "SELECT id FROM users WHERE email = $1 AND username = $2",
        body.email, body.username
    )
    .fetch_optional(&state.db)
    .await;

    match user {
        Ok(Some(row)) => {
            let password_hash = match auth::hash_password(&body.new_password) {
                Ok(h) => h,
                Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to hash password"
                })),
            };

            sqlx::query!(
                "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2",
                password_hash, row.id
            )
            .execute(&state.db)
            .await
            .ok();

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Password updated successfully"
            }))
        }
        _ => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No account found with that email and username combination"
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct GoogleLoginRequest {
    pub credential: String,
}

#[derive(Debug, Deserialize)]
struct GoogleTokenInfo {
    sub: Option<String>,
    email: Option<String>,
    email_verified: Option<String>,
    name: Option<String>,
    aud: Option<String>,
}

pub async fn google_login(
    state: web::Data<AppState>,
    body: web::Json<GoogleLoginRequest>,
) -> HttpResponse {
    let resp = state
        .http_client
        .get("https://oauth2.googleapis.com/tokeninfo")
        .query(&[("id_token", &body.credential)])
        .send()
        .await;

    let token_info: GoogleTokenInfo = match resp {
        Ok(r) if r.status().is_success() => match r.json().await {
            Ok(info) => info,
            Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Google token verification failed"
            })),
        },
        _ => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Google token verification failed"
        })),
    };

    let google_id = match token_info.sub {
        Some(s) => s,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Google token verification failed"
        })),
    };

    let email = match token_info.email {
        Some(e) => e,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Google account has no email"
        })),
    };

    if token_info.email_verified.as_deref() != Some("true") {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Google email not verified"
        }));
    }

    if token_info.aud.as_deref() != Some(&state.google_client_id) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Google token audience mismatch"
        }));
    }

    let display_name = token_info.name.unwrap_or_else(|| {
        email.split('@').next().unwrap_or("User").to_string()
    });

    // 1. Find by google_id
    if let Ok(Some(row)) = sqlx::query!(
        "SELECT id, email, username, display_name, role FROM users WHERE google_id = $1",
        google_id
    )
    .fetch_optional(&state.db)
    .await
    {
        let token = auth::create_token(&row.id, &state.jwt_secret).unwrap();
        return HttpResponse::Ok().json(serde_json::json!({
            "token": token,
            "user": { "id": row.id, "email": row.email, "username": row.username,
                       "display_name": row.display_name, "role": row.role }
        }));
    }

    // 2. Find by email and link
    if let Ok(Some(row)) = sqlx::query!(
        "SELECT id, email, username, display_name, role FROM users WHERE email = $1",
        email
    )
    .fetch_optional(&state.db)
    .await
    {
        sqlx::query!(
            "UPDATE users SET google_id = $1, updated_at = NOW() WHERE id = $2",
            google_id, row.id
        )
        .execute(&state.db)
        .await
        .ok();

        let token = auth::create_token(&row.id, &state.jwt_secret).unwrap();
        return HttpResponse::Ok().json(serde_json::json!({
            "token": token,
            "user": { "id": row.id, "email": row.email, "username": row.username,
                       "display_name": row.display_name, "role": row.role }
        }));
    }

    // 3. Create new user — generate unique username
    let id = Uuid::new_v4().to_string();
    let base_username = email
        .split('@')
        .next()
        .unwrap_or("user")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .to_lowercase();
    let base_username = if base_username.len() < 3 {
        format!("{}user", base_username)
    } else {
        base_username
    };

    let mut username = base_username.clone();
    let mut attempt = 0u32;
    loop {
        let exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE username = $1",
            username
        )
        .fetch_one(&state.db)
        .await
        .unwrap_or(Some(0));

        if exists == Some(0) {
            break;
        }
        attempt += 1;
        username = format!("{}{}", base_username, attempt);
    }

    let result = sqlx::query!(
        "INSERT INTO users (id, email, username, display_name, google_id) VALUES ($1, $2, $3, $4, $5)",
        id, email, username, display_name, google_id
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            let token = auth::create_token(&id, &state.jwt_secret).unwrap();
            HttpResponse::Created().json(serde_json::json!({
                "token": token,
                "user": { "id": id, "email": email, "username": username,
                           "display_name": display_name, "role": "both" }
            }))
        }
        Err(e) => {
            tracing::error!("Google user creation failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create account"
            }))
        }
    }
}

pub async fn me(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    };

    let result = sqlx::query!(
        "SELECT id, email, username, display_name, bio, role, created_at FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    match result {
        Ok(Some(row)) => HttpResponse::Ok().json(serde_json::json!({
            "id": row.id,
            "email": row.email,
            "username": row.username,
            "display_name": row.display_name,
            "bio": row.bio,
            "role": row.role,
            "created_at": row.created_at,
        })),
        _ => HttpResponse::NotFound().json(serde_json::json!({"error": "User not found"})),
    }
}
