use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use super::extract_user_id;

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

pub async fn list(state: web::Data<AppState>) -> HttpResponse {
    let rows = sqlx::query!(
        "SELECT s.id, s.provider_id, s.title, s.description, s.category,
                s.price_cents, s.price_type, s.location, s.created_at,
                u.display_name, u.username,
                COALESCE((SELECT AVG(rv.rating::float) FROM reviews rv
                          WHERE rv.reviewee_id = s.provider_id AND rv.reviewer_role = 'seeker'), 0) as avg_rating,
                (SELECT COUNT(*) FROM reviews rv
                 WHERE rv.reviewee_id = s.provider_id AND rv.reviewer_role = 'seeker') as review_count
         FROM services s
         JOIN users u ON u.id = s.provider_id
         WHERE s.is_active = true
         ORDER BY s.created_at DESC"
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(services) => {
            let result: Vec<serde_json::Value> = services
                .into_iter()
                .map(|r| {
                    let avg = r.avg_rating.unwrap_or(0.0);
                    serde_json::json!({
                        "id": r.id,
                        "provider_id": r.provider_id,
                        "title": r.title,
                        "description": r.description,
                        "category": r.category,
                        "price_cents": r.price_cents,
                        "price_type": r.price_type,
                        "location": r.location,
                        "created_at": r.created_at,
                        "provider_name": r.display_name,
                        "provider_username": r.username,
                        "avg_rating": (avg * 10.0).round() / 10.0,
                        "review_count": r.review_count.unwrap_or(0),
                    })
                })
                .collect();
            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            tracing::error!("Failed to list services: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to fetch services"}))
        }
    }
}

pub async fn create(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<CreateServiceRequest>,
) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
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

    let result = sqlx::query!(
        "INSERT INTO services (id, provider_id, title, description, category, price_cents, price_type, location)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        id, user_id, body.title, body.description, body.category,
        body.price_cents, price_type, location
    )
    .execute(&state.db)
    .await;

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

    let result = sqlx::query!(
        "SELECT s.id, s.provider_id, s.title, s.description, s.category,
                s.price_cents, s.price_type, s.location, s.is_active, s.created_at,
                u.display_name, u.username,
                COALESCE((SELECT AVG(rv.rating::float) FROM reviews rv
                          WHERE rv.reviewee_id = s.provider_id AND rv.reviewer_role = 'seeker'), 0) as avg_rating,
                (SELECT COUNT(*) FROM reviews rv
                 WHERE rv.reviewee_id = s.provider_id AND rv.reviewer_role = 'seeker') as review_count
         FROM services s
         JOIN users u ON u.id = s.provider_id
         WHERE s.id = $1",
        id
    )
    .fetch_optional(&state.db)
    .await;

    match result {
        Ok(Some(r)) => {
            let avg = r.avg_rating.unwrap_or(0.0);
            HttpResponse::Ok().json(serde_json::json!({
                "id": r.id,
                "provider_id": r.provider_id,
                "title": r.title,
                "description": r.description,
                "category": r.category,
                "price_cents": r.price_cents,
                "price_type": r.price_type,
                "location": r.location,
                "is_active": r.is_active,
                "created_at": r.created_at,
                "provider_name": r.display_name,
                "provider_username": r.username,
                "avg_rating": (avg * 10.0).round() / 10.0,
                "review_count": r.review_count.unwrap_or(0),
            }))
        }
        _ => HttpResponse::NotFound().json(serde_json::json!({"error": "Service not found"})),
    }
}

pub async fn mine(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    };

    let rows = sqlx::query!(
        "SELECT id, title, description, category, price_cents, price_type, location, is_active, created_at
         FROM services
         WHERE provider_id = $1
         ORDER BY created_at DESC",
        user_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(services) => {
            let result: Vec<serde_json::Value> = services
                .into_iter()
                .map(|r| serde_json::json!({
                    "id": r.id,
                    "title": r.title,
                    "description": r.description,
                    "category": r.category,
                    "price_cents": r.price_cents,
                    "price_type": r.price_type,
                    "location": r.location,
                    "is_active": r.is_active,
                    "created_at": r.created_at,
                }))
                .collect();
            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            tracing::error!("Failed to fetch my services: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to fetch services"}))
        }
    }
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
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    };

    let service_id = path.into_inner();
    let message = body.message.as_deref().unwrap_or("");
    let id = Uuid::new_v4().to_string();

    // Check service exists and is active
    let service = sqlx::query!(
        "SELECT provider_id FROM services WHERE id = $1 AND is_active = true",
        service_id
    )
    .fetch_optional(&state.db)
    .await;

    match service {
        Ok(Some(s)) => {
            if s.provider_id == user_id {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({"error": "Cannot request your own service"}));
            }
        }
        _ => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Service not found"}));
        }
    }

    let result = sqlx::query!(
        "INSERT INTO service_requests (id, service_id, seeker_id, message) VALUES ($1, $2, $3, $4)",
        id, service_id, user_id, message
    )
    .execute(&state.db)
    .await;

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
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    };

    let rows = sqlx::query!(
        "SELECT r.id, r.service_id, r.message, r.status, r.work_status, r.created_at,
                s.title as service_title, s.provider_id,
                r.seeker_id,
                provider_u.display_name as provider_name,
                seeker_u.display_name as seeker_name,
                (SELECT COUNT(*) FROM reviews rv WHERE rv.request_id = r.id AND rv.reviewer_id = $1) as my_review_count,
                r.decline_reason, r.declined_by
         FROM service_requests r
         JOIN services s ON s.id = r.service_id
         JOIN users provider_u ON provider_u.id = s.provider_id
         JOIN users seeker_u ON seeker_u.id = r.seeker_id
         WHERE r.seeker_id = $1 OR s.provider_id = $1
         ORDER BY r.created_at DESC",
        user_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(requests) => {
            let result: Vec<serde_json::Value> = requests
                .into_iter()
                .map(|r| {
                    let is_provider = user_id == r.provider_id;
                    serde_json::json!({
                        "id": r.id,
                        "service_id": r.service_id,
                        "message": r.message,
                        "status": r.status,
                        "work_status": r.work_status,
                        "created_at": r.created_at,
                        "service_title": r.service_title,
                        "provider_id": r.provider_id,
                        "seeker_id": r.seeker_id,
                        "provider_name": r.provider_name,
                        "seeker_name": r.seeker_name,
                        "has_reviewed": r.my_review_count.unwrap_or(0) > 0,
                        "decline_reason": r.decline_reason,
                        "declined_by": r.declined_by,
                        "my_role": if is_provider { "provider" } else { "seeker" },
                    })
                })
                .collect();
            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            tracing::error!("Failed to fetch requests: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to fetch requests"}))
        }
    }
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
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    };

    let request_id = path.into_inner();
    let status_order = ["not_started", "agreed", "in_progress", "ongoing", "done"];

    let current = sqlx::query!(
        "SELECT r.status, r.work_status, s.provider_id, r.seeker_id
         FROM service_requests r
         JOIN services s ON s.id = r.service_id
         WHERE r.id = $1",
        request_id
    )
    .fetch_optional(&state.db)
    .await;

    match current {
        Ok(Some(row)) => {
            let is_provider = user_id == row.provider_id;
            let is_seeker = user_id == row.seeker_id;

            if !is_provider && !is_seeker {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "You are not part of this request"}));
            }
            if row.status != "accepted" {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({"error": "Request must be accepted to update work status"}));
            }

            let current_idx = status_order.iter().position(|&s| s == row.work_status.as_str());
            let target_idx = status_order.iter().position(|&s| s == body.work_status.as_str());

            let (current_idx, target_idx) = match (current_idx, target_idx) {
                (Some(c), Some(t)) => (c, t),
                _ => return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid work status '{}'", body.work_status)
                })),
            };

            if target_idx <= current_idx {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Cannot transition from '{}' to '{}'", row.work_status, body.work_status)
                }));
            }

            if is_seeker && !(row.work_status == "not_started" && body.work_status == "agreed") {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "Seekers can only accept the offer"}));
            }

            sqlx::query!(
                "UPDATE service_requests SET work_status = $1, updated_at = NOW() WHERE id = $2",
                body.work_status, request_id
            )
            .execute(&state.db)
            .await
            .ok();

            HttpResponse::Ok().json(serde_json::json!({"message": "Work status updated"}))
        }
        _ => HttpResponse::NotFound()
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
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    };

    let request_id = path.into_inner();
    let valid_statuses = ["accepted", "declined", "completed", "cancelled"];

    if !valid_statuses.contains(&body.status.as_str()) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Status must be one of: {}", valid_statuses.join(", "))
        }));
    }

    let current = sqlx::query!(
        "SELECT s.provider_id, r.seeker_id, r.status, r.work_status
         FROM service_requests r
         JOIN services s ON s.id = r.service_id
         WHERE r.id = $1",
        request_id
    )
    .fetch_optional(&state.db)
    .await;

    match current {
        Ok(Some(row)) => {
            let is_provider = row.provider_id == user_id;
            let is_seeker = row.seeker_id == user_id;

            if !is_provider && !is_seeker {
                return HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "You are not part of this request"}));
            }

            match body.status.as_str() {
                "declined" => {
                    if row.status != "pending" && row.status != "accepted" {
                        return HttpResponse::BadRequest()
                            .json(serde_json::json!({"error": "Can only decline pending or accepted requests"}));
                    }
                    if row.status == "accepted"
                        && row.work_status != "not_started"
                        && row.work_status != "agreed"
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
            let declined_by: Option<String> = if body.status == "declined" || body.status == "cancelled" {
                Some(if is_provider { "provider".to_string() } else { "seeker".to_string() })
            } else {
                None
            };

            sqlx::query!(
                "UPDATE service_requests SET status = $1, decline_reason = $2, declined_by = $3, updated_at = NOW() WHERE id = $4",
                body.status, reason, declined_by, request_id
            )
            .execute(&state.db)
            .await
            .ok();

            HttpResponse::Ok().json(serde_json::json!({"message": "Status updated"}))
        }
        _ => HttpResponse::NotFound()
            .json(serde_json::json!({"error": "Request not found"})),
    }
}
