use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use super::extract_user_id;

#[derive(Debug, Deserialize)]
pub struct GetMessagesQuery {
    pub after: Option<String>,
}

pub async fn get_messages(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<GetMessagesQuery>,
) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    };

    let request_id = path.into_inner();

    // Verify user is a party to this request
    let party = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM service_requests r
         JOIN services s ON s.id = r.service_id
         WHERE r.id = $1 AND (r.seeker_id = $2 OR s.provider_id = $2)",
        request_id, user_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0));

    if party == Some(0) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "You are not part of this request"}));
    }

    // Mark messages from other party as read
    sqlx::query!(
        "UPDATE messages SET read_at = NOW()
         WHERE request_id = $1 AND sender_id != $2 AND read_at IS NULL",
        request_id, user_id
    )
    .execute(&state.db)
    .await
    .ok();

    // Fetch messages — use a single query with optional timestamp filter
    let after_dt: Option<chrono::DateTime<chrono::Utc>> = query.after
        .as_deref()
        .and_then(|s| s.parse().ok());

    let messages = sqlx::query!(
        "SELECT m.id, m.sender_id, m.body, m.created_at, u.display_name
         FROM messages m
         JOIN users u ON u.id = m.sender_id
         WHERE m.request_id = $1
           AND ($2::timestamptz IS NULL OR m.created_at > $2::timestamptz)
         ORDER BY m.created_at ASC, m.id ASC",
        request_id, after_dt
    )
    .fetch_all(&state.db)
    .await;

    match messages {
        Ok(rows) => {
            let result: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|r| serde_json::json!({
                    "id": r.id,
                    "sender_id": r.sender_id,
                    "body": r.body,
                    "created_at": r.created_at,
                    "sender_name": r.display_name,
                }))
                .collect();
            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            tracing::error!("Failed to fetch messages: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to fetch messages"}))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SendMessageBody {
    pub body: String,
}

pub async fn send_message(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<SendMessageBody>,
) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    };

    let request_id = path.into_inner();

    if body.body.is_empty() || body.body.len() > 2000 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Message must be 1-2000 characters"}));
    }

    // Verify party
    let party = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM service_requests r
         JOIN services s ON s.id = r.service_id
         WHERE r.id = $1 AND (r.seeker_id = $2 OR s.provider_id = $2)",
        request_id, user_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0));

    if party == Some(0) {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"error": "You are not part of this request"}));
    }

    let id = Uuid::new_v4().to_string();

    let result = sqlx::query!(
        "INSERT INTO messages (id, request_id, sender_id, body) VALUES ($1, $2, $3, $4)",
        id, request_id, user_id, body.body
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({
            "id": id,
            "message": "Message sent"
        })),
        Err(e) => {
            tracing::error!("Failed to send message: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to send message"}))
        }
    }
}

pub async fn unread_count(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    };

    let count = sqlx::query_scalar!(
        "SELECT COUNT(*)
         FROM messages m
         JOIN service_requests r ON r.id = m.request_id
         JOIN services s ON s.id = r.service_id
         WHERE m.read_at IS NULL
           AND m.sender_id != $1
           AND (r.seeker_id = $1 OR s.provider_id = $1)",
        user_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0));

    HttpResponse::Ok().json(serde_json::json!({"count": count.unwrap_or(0)}))
}