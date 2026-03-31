use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::auth;
use crate::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateServiceRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1))]
    pub description: String,
    #[validate(length(min = 1, max = 50))]
    pub category: String,
    pub price_cents: Option<i64>,
    pub price_type: Option<String>,
    pub location: Option<String>,
}

fn extract_user_id(req: &HttpRequest, jwt_secret: &str) -> Result<String, HttpResponse> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            HttpResponse::Unauthorized().json(serde_json::json!({"error": "Not authenticated"}))
        })?;

    auth::validate_token(token, jwt_secret)
        .map(|c| c.sub)
        .map_err(|_| {
            HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Invalid or expired token"}))
        })
}

pub async fn list(state: web::Data<AppState>) -> HttpResponse {
    let db = state.db.lock().unwrap();

    let mut stmt = db
        .prepare(
            "SELECT s.id, s.provider_id, s.title, s.description, s.category,
                    s.price_cents, s.price_type, s.location, s.created_at,
                    u.display_name, u.username
             FROM services s
             JOIN users u ON u.id = s.provider_id
             WHERE s.is_active = 1
             ORDER BY s.created_at DESC",
        )
        .unwrap();

    let services: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "provider_id": row.get::<_, String>(1)?,
                "title": row.get::<_, String>(2)?,
                "description": row.get::<_, String>(3)?,
                "category": row.get::<_, String>(4)?,
                "price_cents": row.get::<_, Option<i64>>(5)?,
                "price_type": row.get::<_, String>(6)?,
                "location": row.get::<_, String>(7)?,
                "created_at": row.get::<_, String>(8)?,
                "provider_name": row.get::<_, String>(9)?,
                "provider_username": row.get::<_, String>(10)?,
            }))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    HttpResponse::Ok().json(services)
}

pub async fn create(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<CreateServiceRequest>,
) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    if let Err(errors) = body.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    let id = Uuid::new_v4().to_string();
    let price_type = body.price_type.as_deref().unwrap_or("negotiable");
    let location = body.location.as_deref().unwrap_or("");
    let db = state.db.lock().unwrap();

    let result = db.execute(
        "INSERT INTO services (id, provider_id, title, description, category, price_cents, price_type, location)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![id, user_id, body.title, body.description, body.category, body.price_cents, price_type, location],
    );

    match result {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({
            "id": id,
            "message": "Service created"
        })),
        Err(e) => {
            tracing::error!("Failed to create service: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to create service"}))
        }
    }
}

pub async fn get(state: web::Data<AppState>, path: web::Path<String>) -> HttpResponse {
    let id = path.into_inner();
    let db = state.db.lock().unwrap();

    let result = db.query_row(
        "SELECT s.id, s.provider_id, s.title, s.description, s.category,
                s.price_cents, s.price_type, s.location, s.is_active, s.created_at,
                u.display_name, u.username
         FROM services s
         JOIN users u ON u.id = s.provider_id
         WHERE s.id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "provider_id": row.get::<_, String>(1)?,
                "title": row.get::<_, String>(2)?,
                "description": row.get::<_, String>(3)?,
                "category": row.get::<_, String>(4)?,
                "price_cents": row.get::<_, Option<i64>>(5)?,
                "price_type": row.get::<_, String>(6)?,
                "location": row.get::<_, String>(7)?,
                "is_active": row.get::<_, bool>(8)?,
                "created_at": row.get::<_, String>(9)?,
                "provider_name": row.get::<_, String>(10)?,
                "provider_username": row.get::<_, String>(11)?,
            }))
        },
    );

    match result {
        Ok(service) => HttpResponse::Ok().json(service),
        Err(_) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Service not found"}))
        }
    }
}

pub async fn mine(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let db = state.db.lock().unwrap();

    let mut stmt = db
        .prepare(
            "SELECT id, title, description, category, price_cents, price_type, location, is_active, created_at
             FROM services
             WHERE provider_id = ?1
             ORDER BY created_at DESC",
        )
        .unwrap();

    let services: Vec<serde_json::Value> = stmt
        .query_map(rusqlite::params![user_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "description": row.get::<_, String>(2)?,
                "category": row.get::<_, String>(3)?,
                "price_cents": row.get::<_, Option<i64>>(4)?,
                "price_type": row.get::<_, String>(5)?,
                "location": row.get::<_, String>(6)?,
                "is_active": row.get::<_, bool>(7)?,
                "created_at": row.get::<_, String>(8)?,
            }))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    HttpResponse::Ok().json(services)
}

#[derive(Debug, Deserialize)]
pub struct RequestServiceBody {
    pub message: Option<String>,
}

pub async fn request_service(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<RequestServiceBody>,
) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let service_id = path.into_inner();
    let message = body.message.as_deref().unwrap_or("");
    let id = Uuid::new_v4().to_string();
    let db = state.db.lock().unwrap();

    // Check service exists and is active
    let exists = db
        .query_row(
            "SELECT provider_id FROM services WHERE id = ?1 AND is_active = 1",
            rusqlite::params![service_id],
            |row| row.get::<_, String>(0),
        );

    match exists {
        Ok(provider_id) => {
            if provider_id == user_id {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({"error": "Cannot request your own service"}));
            }
        }
        Err(_) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Service not found"}));
        }
    }

    let result = db.execute(
        "INSERT INTO service_requests (id, service_id, seeker_id, message)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, service_id, user_id, message],
    );

    match result {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({
            "id": id,
            "message": "Request sent"
        })),
        Err(e) => {
            tracing::error!("Failed to create request: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to send request"}))
        }
    }
}

pub async fn my_requests(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let db = state.db.lock().unwrap();

    let mut stmt = db
        .prepare(
            "SELECT r.id, r.service_id, r.message, r.status, r.created_at,
                    s.title as service_title
             FROM service_requests r
             JOIN services s ON s.id = r.service_id
             WHERE r.seeker_id = ?1 OR s.provider_id = ?1
             ORDER BY r.created_at DESC",
        )
        .unwrap();

    let requests: Vec<serde_json::Value> = stmt
        .query_map(rusqlite::params![user_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "service_id": row.get::<_, String>(1)?,
                "message": row.get::<_, String>(2)?,
                "status": row.get::<_, String>(3)?,
                "created_at": row.get::<_, String>(4)?,
                "service_title": row.get::<_, String>(5)?,
            }))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    HttpResponse::Ok().json(requests)
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusBody {
    pub status: String,
}

pub async fn update_request_status(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateStatusBody>,
) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let request_id = path.into_inner();
    let valid_statuses = ["accepted", "declined", "completed", "cancelled"];
    if !valid_statuses.contains(&body.status.as_str()) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Status must be one of: {}", valid_statuses.join(", "))
        }));
    }

    let db = state.db.lock().unwrap();

    // Verify the user is the provider of the service this request is for
    let authorized = db.query_row(
        "SELECT s.provider_id, r.seeker_id
         FROM service_requests r
         JOIN services s ON s.id = r.service_id
         WHERE r.id = ?1",
        rusqlite::params![request_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    );

    match authorized {
        Ok((provider_id, seeker_id)) => {
            let is_provider = provider_id == user_id;
            let is_seeker = seeker_id == user_id;

            // Provider can accept/decline/complete; seeker can cancel
            if body.status == "cancelled" && !is_seeker {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "Only the requester can cancel"}));
            }
            if body.status != "cancelled" && !is_provider {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "Only the provider can update this status"}));
            }
        }
        Err(_) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Request not found"}));
        }
    }

    db.execute(
        "UPDATE service_requests SET status = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?2",
        rusqlite::params![body.status, request_id],
    )
    .unwrap();

    HttpResponse::Ok().json(serde_json::json!({"message": "Status updated"}))
}
