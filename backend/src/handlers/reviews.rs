use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth;
use crate::AppState;

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

#[derive(Debug, Deserialize)]
pub struct SubmitReviewBody {
    pub rating: i32,
    pub comment: Option<String>,
}

pub async fn submit(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<SubmitReviewBody>,
) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    if body.rating < 1 || body.rating > 5 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Rating must be between 1 and 5"}));
    }

    let request_id = path.into_inner();
    let db = state.db.lock().unwrap();

    // Get request details: seeker_id, provider_id, work_status
    let request_info = db.query_row(
        "SELECT r.seeker_id, s.provider_id, r.work_status
         FROM service_requests r
         JOIN services s ON s.id = r.service_id
         WHERE r.id = ?1",
        rusqlite::params![request_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        },
    );

    let (seeker_id, provider_id, work_status) = match request_info {
        Ok(info) => info,
        Err(_) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Request not found"}));
        }
    };

    // Check work_status allows reviews
    if work_status != "in_progress" && work_status != "ongoing" && work_status != "done" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Reviews are only available once work is in progress"
        }));
    }

    // Determine reviewer role and reviewee
    let (reviewer_role, reviewee_id) = if user_id == seeker_id {
        ("seeker", provider_id)
    } else if user_id == provider_id {
        ("provider", seeker_id)
    } else {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "You are not part of this request"}));
    };

    let comment = body.comment.as_deref().unwrap_or("");
    let id = Uuid::new_v4().to_string();

    let result = db.execute(
        "INSERT INTO reviews (id, request_id, reviewer_id, reviewee_id, reviewer_role, rating, comment)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, request_id, user_id, reviewee_id, reviewer_role, body.rating, comment],
    );

    match result {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({
            "id": id,
            "message": "Review submitted"
        })),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("UNIQUE") {
                HttpResponse::Conflict().json(serde_json::json!({
                    "error": "You have already submitted a review for this request"
                }))
            } else {
                tracing::error!("Failed to submit review: {}", msg);
                HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Failed to submit review"}))
            }
        }
    }
}

pub async fn for_request(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let request_id = path.into_inner();
    let db = state.db.lock().unwrap();

    // Verify user is part of this request
    let is_party = db.query_row(
        "SELECT 1 FROM service_requests r
         JOIN services s ON s.id = r.service_id
         WHERE r.id = ?1 AND (r.seeker_id = ?2 OR s.provider_id = ?2)",
        rusqlite::params![request_id, user_id],
        |_| Ok(()),
    );

    if is_party.is_err() {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "You are not part of this request"}));
    }

    let mut stmt = db
        .prepare(
            "SELECT rv.id, rv.reviewer_role, rv.rating, rv.comment, rv.created_at,
                    u.display_name as reviewer_name
             FROM reviews rv
             JOIN users u ON u.id = rv.reviewer_id
             WHERE rv.request_id = ?1
             ORDER BY rv.created_at",
        )
        .unwrap();

    let reviews: Vec<serde_json::Value> = stmt
        .query_map(rusqlite::params![request_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "reviewer_role": row.get::<_, String>(1)?,
                "rating": row.get::<_, i32>(2)?,
                "comment": row.get::<_, String>(3)?,
                "created_at": row.get::<_, String>(4)?,
                "reviewer_name": row.get::<_, String>(5)?,
            }))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    HttpResponse::Ok().json(reviews)
}

pub async fn user_ratings(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let user_id = path.into_inner();
    let db = state.db.lock().unwrap();

    // Overall
    let overall = db.query_row(
        "SELECT COALESCE(AVG(rating), 0), COUNT(*) FROM reviews WHERE reviewee_id = ?1",
        rusqlite::params![user_id],
        |row| Ok((row.get::<_, f64>(0)?, row.get::<_, i64>(1)?)),
    );

    // As provider (reviewed by seekers)
    let as_provider = db.query_row(
        "SELECT COALESCE(AVG(rating), 0), COUNT(*) FROM reviews WHERE reviewee_id = ?1 AND reviewer_role = 'seeker'",
        rusqlite::params![user_id],
        |row| Ok((row.get::<_, f64>(0)?, row.get::<_, i64>(1)?)),
    );

    // As seeker (reviewed by providers)
    let as_seeker = db.query_row(
        "SELECT COALESCE(AVG(rating), 0), COUNT(*) FROM reviews WHERE reviewee_id = ?1 AND reviewer_role = 'provider'",
        rusqlite::params![user_id],
        |row| Ok((row.get::<_, f64>(0)?, row.get::<_, i64>(1)?)),
    );

    let (avg, count) = overall.unwrap_or((0.0, 0));
    let (prov_avg, prov_count) = as_provider.unwrap_or((0.0, 0));
    let (seek_avg, seek_count) = as_seeker.unwrap_or((0.0, 0));

    HttpResponse::Ok().json(serde_json::json!({
        "avg_rating": (avg * 10.0).round() / 10.0,
        "review_count": count,
        "as_provider": {
            "avg": (prov_avg * 10.0).round() / 10.0,
            "count": prov_count
        },
        "as_seeker": {
            "avg": (seek_avg * 10.0).round() / 10.0,
            "count": seek_count
        }
    }))
}
