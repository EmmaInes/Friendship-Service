use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use super::extract_user_id;

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
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    };

    if body.rating < 1 || body.rating > 5 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Rating must be between 1 and 5"}));
    }

    let request_id = path.into_inner();

    let request_info = sqlx::query!(
        "SELECT r.seeker_id, s.provider_id, r.work_status
         FROM service_requests r
         JOIN services s ON s.id = r.service_id
         WHERE r.id = $1",
        request_id
    )
    .fetch_optional(&state.db)
    .await;

    let row = match request_info {
        Ok(Some(r)) => r,
        _ => return HttpResponse::NotFound()
            .json(serde_json::json!({"error": "Request not found"})),
    };

    if row.work_status != "in_progress" && row.work_status != "ongoing" && row.work_status != "done" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Reviews are only available once work is in progress"
        }));
    }

    let (reviewer_role, reviewee_id) = if user_id == row.seeker_id {
        ("seeker", row.provider_id)
    } else if user_id == row.provider_id {
        ("provider", row.seeker_id)
    } else {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "You are not part of this request"}));
    };

    let comment = body.comment.as_deref().unwrap_or("");
    let id = Uuid::new_v4().to_string();

    let result = sqlx::query!(
        "INSERT INTO reviews (id, request_id, reviewer_id, reviewee_id, reviewer_role, rating, comment)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        id, request_id, user_id, reviewee_id, reviewer_role, body.rating, comment
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({
            "id": id,
            "message": "Review submitted"
        })),
        Err(sqlx::Error::Database(e)) if e.constraint().is_some() => {
            HttpResponse::Conflict().json(serde_json::json!({
                "error": "You have already submitted a review for this request"
            }))
        }
        Err(e) => {
            tracing::error!("Failed to submit review: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to submit review"}))
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
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    };

    let request_id = path.into_inner();

    let is_party = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM service_requests r
         JOIN services s ON s.id = r.service_id
         WHERE r.id = $1 AND (r.seeker_id = $2 OR s.provider_id = $2)",
        request_id, user_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0));

    if is_party == Some(0) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "You are not part of this request"}));
    }

    let rows = sqlx::query!(
        "SELECT rv.id, rv.reviewer_role, rv.rating, rv.comment, rv.created_at,
                u.display_name as reviewer_name
         FROM reviews rv
         JOIN users u ON u.id = rv.reviewer_id
         WHERE rv.request_id = $1
         ORDER BY rv.created_at",
        request_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(reviews) => {
            let result: Vec<serde_json::Value> = reviews
                .into_iter()
                .map(|r| serde_json::json!({
                    "id": r.id,
                    "reviewer_role": r.reviewer_role,
                    "rating": r.rating,
                    "comment": r.comment,
                    "created_at": r.created_at,
                    "reviewer_name": r.reviewer_name,
                }))
                .collect();
            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            tracing::error!("Failed to fetch reviews: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to fetch reviews"}))
        }
    }
}

pub async fn user_ratings(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let user_id = path.into_inner();

    let overall = sqlx::query!(
        "SELECT COALESCE(AVG(rating::float), 0) as avg, COUNT(*) as count
         FROM reviews WHERE reviewee_id = $1",
        user_id
    )
    .fetch_one(&state.db)
    .await;

    let as_provider = sqlx::query!(
        "SELECT COALESCE(AVG(rating::float), 0) as avg, COUNT(*) as count
         FROM reviews WHERE reviewee_id = $1 AND reviewer_role = 'seeker'",
        user_id
    )
    .fetch_one(&state.db)
    .await;

    let as_seeker = sqlx::query!(
        "SELECT COALESCE(AVG(rating::float), 0) as avg, COUNT(*) as count
         FROM reviews WHERE reviewee_id = $1 AND reviewer_role = 'provider'",
        user_id
    )
    .fetch_one(&state.db)
    .await;

    let (avg, count) = overall.map(|r| (r.avg.unwrap_or(0.0), r.count.unwrap_or(0))).unwrap_or((0.0, 0));
    let (prov_avg, prov_count) = as_provider.map(|r| (r.avg.unwrap_or(0.0), r.count.unwrap_or(0))).unwrap_or((0.0, 0));
    let (seek_avg, seek_count) = as_seeker.map(|r| (r.avg.unwrap_or(0.0), r.count.unwrap_or(0))).unwrap_or((0.0, 0));

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
