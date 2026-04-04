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
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        },
    );

    match result {
        Ok((id, hash, email, username, display_name, role)) => {
            let hash = match hash {
                Some(h) => h,
                None => {
                    // Google-only user, cannot login with password
                    return HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "This account uses Google Sign-In"
                    }));
                }
            };
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

    let db = state.db.lock().unwrap();

    let user_id = db.query_row(
        "SELECT id FROM users WHERE email = ?1 AND username = ?2",
        rusqlite::params![body.email, body.username],
        |row| row.get::<_, String>(0),
    );

    match user_id {
        Ok(id) => {
            let password_hash = match auth::hash_password(&body.new_password) {
                Ok(h) => h,
                Err(_) => {
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to hash password"
                    }))
                }
            };

            db.execute(
                "UPDATE users SET password_hash = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?2",
                rusqlite::params![password_hash, id],
            )
            .unwrap();

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Password updated successfully"
            }))
        }
        Err(_) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No account found with that email and username combination"
        })),
    }
}

// --- Google OAuth ---

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
    // Verify the Google ID token via Google's tokeninfo endpoint
    let resp = state
        .http_client
        .get("https://oauth2.googleapis.com/tokeninfo")
        .query(&[("id_token", &body.credential)])
        .send()
        .await;

    let token_info: GoogleTokenInfo = match resp {
        Ok(r) if r.status().is_success() => match r.json().await {
            Ok(info) => info,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Google token verification failed"
                }));
            }
        },
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Google token verification failed"
            }));
        }
    };

    // Validate token
    let google_id = match &token_info.sub {
        Some(sub) => sub.clone(),
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Google token verification failed"
            }));
        }
    };

    let email = match &token_info.email {
        Some(e) => e.clone(),
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Google account has no email"
            }));
        }
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

    let display_name = token_info.name.unwrap_or_else(|| email.split('@').next().unwrap_or("User").to_string());

    let db = state.db.lock().unwrap();

    // 1. Try to find user by google_id
    let by_google = db.query_row(
        "SELECT id, email, username, display_name, role FROM users WHERE google_id = ?1",
        rusqlite::params![google_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        },
    );

    if let Ok((id, email, username, display_name, role)) = by_google {
        let token = auth::create_token(&id, &state.jwt_secret).unwrap();
        return HttpResponse::Ok().json(serde_json::json!({
            "token": token,
            "user": { "id": id, "email": email, "username": username, "display_name": display_name, "role": role }
        }));
    }

    // 2. Try to find user by email (link existing account)
    let by_email = db.query_row(
        "SELECT id, email, username, display_name, role FROM users WHERE email = ?1",
        rusqlite::params![email],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        },
    );

    if let Ok((id, email, username, display_name, role)) = by_email {
        // Link Google account
        db.execute(
            "UPDATE users SET google_id = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?2",
            rusqlite::params![google_id, id],
        )
        .ok();
        let token = auth::create_token(&id, &state.jwt_secret).unwrap();
        return HttpResponse::Ok().json(serde_json::json!({
            "token": token,
            "user": { "id": id, "email": email, "username": username, "display_name": display_name, "role": role }
        }));
    }

    // 3. Create new user
    let id = Uuid::new_v4().to_string();

    // Generate username from email prefix, handle collisions
    let base_username = email.split('@').next().unwrap_or("user")
        .chars().filter(|c| c.is_alphanumeric() || *c == '_').collect::<String>()
        .to_lowercase();
    let base_username = if base_username.len() < 3 { format!("{}user", base_username) } else { base_username };

    let mut username = base_username.clone();
    let mut attempt = 0;
    loop {
        let exists = db.query_row(
            "SELECT 1 FROM users WHERE username = ?1",
            rusqlite::params![username],
            |_| Ok(()),
        );
        if exists.is_err() {
            break; // username is available
        }
        attempt += 1;
        username = format!("{}{}", base_username, attempt);
    }

    let result = db.execute(
        "INSERT INTO users (id, email, username, display_name, google_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, email, username, display_name, google_id],
    );

    match result {
        Ok(_) => {
            let token = auth::create_token(&id, &state.jwt_secret).unwrap();
            HttpResponse::Created().json(serde_json::json!({
                "token": token,
                "user": { "id": id, "email": email, "username": username, "display_name": display_name, "role": "both" }
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
