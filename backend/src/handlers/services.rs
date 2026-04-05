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
                    u.display_name, u.username,
                    COALESCE((SELECT AVG(rv.rating) FROM reviews rv WHERE rv.reviewee_id = s.provider_id AND rv.reviewer_role = 'seeker'), 0) as avg_rating,
                    (SELECT COUNT(*) FROM reviews rv WHERE rv.reviewee_id = s.provider_id AND rv.reviewer_role = 'seeker') as review_count
             FROM services s
             JOIN users u ON u.id = s.provider_id
             WHERE s.is_active = 1
             ORDER BY s.created_at DESC",
        )
        .unwrap();

    let services: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            let avg: f64 = row.get(11)?;
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
                "avg_rating": (avg * 10.0).round() / 10.0,
                "review_count": row.get::<_, i64>(12)?,
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
                u.display_name, u.username,
                COALESCE((SELECT AVG(rv.rating) FROM reviews rv WHERE rv.reviewee_id = s.provider_id AND rv.reviewer_role = 'seeker'), 0),
                (SELECT COUNT(*) FROM reviews rv WHERE rv.reviewee_id = s.provider_id AND rv.reviewer_role = 'seeker')
         FROM services s
         JOIN users u ON u.id = s.provider_id
         WHERE s.id = ?1",
        rusqlite::params![id],
        |row| {
            let avg: f64 = row.get(12)?;
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
                "avg_rating": (avg * 10.0).round() / 10.0,
                "review_count": row.get::<_, i64>(13)?,
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
            "SELECT r.id, r.service_id, r.message, r.status, r.work_status, r.created_at,
                    s.title as service_title, s.provider_id,
                    r.seeker_id,
                    provider_u.display_name as provider_name,
                    seeker_u.display_name as seeker_name,
                    (SELECT COUNT(*) FROM reviews rv WHERE rv.request_id = r.id AND rv.reviewer_id = ?1) as my_review_count,
                    r.decline_reason, r.declined_by
             FROM service_requests r
             JOIN services s ON s.id = r.service_id
             JOIN users provider_u ON provider_u.id = s.provider_id
             JOIN users seeker_u ON seeker_u.id = r.seeker_id
             WHERE r.seeker_id = ?1 OR s.provider_id = ?1
             ORDER BY r.created_at DESC",
        )
        .unwrap();

    let requests: Vec<serde_json::Value> = stmt
        .query_map(rusqlite::params![user_id], |row| {
            let provider_id: String = row.get(7)?;
            let seeker_id: String = row.get(8)?;
            let is_provider = user_id == provider_id;
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "service_id": row.get::<_, String>(1)?,
                "message": row.get::<_, String>(2)?,
                "status": row.get::<_, String>(3)?,
                "work_status": row.get::<_, String>(4)?,
                "created_at": row.get::<_, String>(5)?,
                "service_title": row.get::<_, String>(6)?,
                "provider_id": provider_id,
                "seeker_id": seeker_id,
                "provider_name": row.get::<_, String>(9)?,
                "seeker_name": row.get::<_, String>(10)?,
                "has_reviewed": row.get::<_, i64>(11)? > 0,
                "decline_reason": row.get::<_, String>(12)?,
                "declined_by": row.get::<_, Option<String>>(13)?,
                "my_role": if is_provider { "provider" } else { "seeker" },
            }))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    HttpResponse::Ok().json(requests)
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkStatusBody {
    pub work_status: String,
}

pub async fn update_work_status(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateWorkStatusBody>,
) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let request_id = path.into_inner();
    let valid_transitions = [
        ("not_started", "agreed"),
        ("agreed", "in_progress"),
        ("in_progress", "ongoing"),
        ("ongoing", "done"),
    ];

    let db = state.db.lock().unwrap();

    // Verify party and get current state
    let current = db.query_row(
        "SELECT r.status, r.work_status, s.provider_id, r.seeker_id
         FROM service_requests r
         JOIN services s ON s.id = r.service_id
         WHERE r.id = ?1",
        rusqlite::params![request_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?)),
    );

    match current {
        Ok((status, current_ws, provider_id, seeker_id)) => {
            let is_provider = user_id == provider_id;
            let is_seeker = user_id == seeker_id;

            if !is_provider && !is_seeker {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "You are not part of this request"}));
            }
            if status != "accepted" {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({"error": "Request must be accepted to update work status"}));
            }

            // Seeker can only accept offer: not_started -> in_progress
            // Provider can do all transitions
            let is_valid = valid_transitions
                .iter()
                .any(|(from, to)| *from == current_ws && *to == body.work_status);

            if !is_valid {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Cannot transition from '{}' to '{}'", current_ws, body.work_status)
                }));
            }

            if is_seeker && !(current_ws == "not_started" && body.work_status == "agreed") {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "Seekers can only accept the offer"}));
            }

            if !is_valid {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Cannot transition from '{}' to '{}'", current_ws, body.work_status)
                }));
            }

            db.execute(
                "UPDATE service_requests SET work_status = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?2",
                rusqlite::params![body.work_status, request_id],
            )
            .unwrap();

            HttpResponse::Ok().json(serde_json::json!({"message": "Work status updated"}))
        }
        Err(_) => HttpResponse::NotFound()
            .json(serde_json::json!({"error": "Request not found"})),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusBody {
    pub status: String,
    pub reason: Option<String>,
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

    let authorized = db.query_row(
        "SELECT s.provider_id, r.seeker_id, r.status, r.work_status
         FROM service_requests r
         JOIN services s ON s.id = r.service_id
         WHERE r.id = ?1",
        rusqlite::params![request_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?)),
    );

    match authorized {
        Ok((provider_id, seeker_id, current_status, work_status)) => {
            let is_provider = provider_id == user_id;
            let is_seeker = seeker_id == user_id;

            if !is_provider && !is_seeker {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "You are not part of this request"}));
            }

            // Both can decline (with reason)
            // Provider can also accept/complete
            // Seeker can also cancel
            match body.status.as_str() {
                "declined" => {
                    if current_status != "pending" && current_status != "accepted" {
                        return HttpResponse::BadRequest()
                            .json(serde_json::json!({"error": "Can only decline pending or accepted requests"}));
                    }
                    // Once work is in_progress or beyond, no more declines
                    if current_status == "accepted"
                        && work_status != "not_started"
                        && work_status != "agreed"
                    {
                        return HttpResponse::BadRequest()
                            .json(serde_json::json!({"error": "Cannot decline once work is in progress"}));
                    }
                }
                "cancelled" => {
                    if !is_seeker {
                        return HttpResponse::Forbidden()
                            .json(serde_json::json!({"error": "Only the requester can cancel"}));
                    }
                }
                "accepted" | "completed" => {
                    if !is_provider {
                        return HttpResponse::Forbidden()
                            .json(serde_json::json!({"error": "Only the provider can update this status"}));
                    }
                }
                _ => {}
            }

            let reason = body.reason.as_deref().unwrap_or("");
            let declined_by = if body.status == "declined" || body.status == "cancelled" {
                Some(if is_provider { "provider" } else { "seeker" })
            } else {
                None
            };

            db.execute(
                "UPDATE service_requests SET status = ?1, decline_reason = ?2, declined_by = ?3, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?4",
                rusqlite::params![body.status, reason, declined_by, request_id],
            )
            .unwrap();
        }
        Err(_) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Request not found"}));
        }
    }

    HttpResponse::Ok().json(serde_json::json!({"message": "Status updated"}))
}
