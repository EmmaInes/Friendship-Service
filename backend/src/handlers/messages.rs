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

/// Verify user is seeker or provider for this request. Returns (seeker_id, provider_id).
fn verify_party(
    db: &rusqlite::Connection,
    request_id: &str,
    user_id: &str,
) -> Result<(String, String), HttpResponse> {
    let result = db.query_row(
        "SELECT r.seeker_id, s.provider_id
         FROM service_requests r
         JOIN services s ON s.id = r.service_id
         WHERE r.id = ?1",
        rusqlite::params![request_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    );

    match result {
        Ok((seeker_id, provider_id)) => {
            if user_id != seeker_id && user_id != provider_id {
                Err(HttpResponse::Forbidden()
                    .json(serde_json::json!({"error": "You are not part of this request"})))
            } else {
                Ok((seeker_id, provider_id))
            }
        }
        Err(_) => Err(HttpResponse::NotFound()
            .json(serde_json::json!({"error": "Request not found"}))),
    }
}

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
        Err(resp) => return resp,
    };

    let request_id = path.into_inner();
    let db = state.db.lock().unwrap();

    if let Err(resp) = verify_party(&db, &request_id, &user_id) {
        return resp;
    }

    // Mark messages from the other party as read
    db.execute(
        "UPDATE messages SET read_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE request_id = ?1 AND sender_id != ?2 AND read_at IS NULL",
        rusqlite::params![request_id, user_id],
    )
    .ok();

    // Fetch messages
    let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(ref after) = query.after {
        (
            "SELECT m.id, m.sender_id, m.body, m.created_at, u.display_name
             FROM messages m
             JOIN users u ON u.id = m.sender_id
             WHERE m.request_id = ?1 AND m.created_at > ?2
             ORDER BY m.created_at ASC, m.id ASC",
            vec![Box::new(request_id.clone()), Box::new(after.clone())],
        )
    } else {
        (
            "SELECT m.id, m.sender_id, m.body, m.created_at, u.display_name
             FROM messages m
             JOIN users u ON u.id = m.sender_id
             WHERE m.request_id = ?1
             ORDER BY m.created_at ASC, m.id ASC",
            vec![Box::new(request_id.clone())],
        )
    };

    let mut stmt = db.prepare(sql).unwrap();
    let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let messages: Vec<serde_json::Value> = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "sender_id": row.get::<_, String>(1)?,
                "body": row.get::<_, String>(2)?,
                "created_at": row.get::<_, String>(3)?,
                "sender_name": row.get::<_, String>(4)?,
            }))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    HttpResponse::Ok().json(messages)
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
        Err(resp) => return resp,
    };

    let request_id = path.into_inner();

    if body.body.is_empty() || body.body.len() > 2000 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Message must be 1-2000 characters"}));
    }

    let db = state.db.lock().unwrap();

    if let Err(resp) = verify_party(&db, &request_id, &user_id) {
        return resp;
    }

    let id = Uuid::new_v4().to_string();

    let result = db.execute(
        "INSERT INTO messages (id, request_id, sender_id, body)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, request_id, user_id, body.body],
    );

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

pub async fn unread_count(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let db = state.db.lock().unwrap();

    // Count unread messages where user is a party but NOT the sender
    let count: i64 = db
        .query_row(
            "SELECT COUNT(*)
             FROM messages m
             JOIN service_requests r ON r.id = m.request_id
             JOIN services s ON s.id = r.service_id
             WHERE m.read_at IS NULL
               AND m.sender_id != ?1
               AND (r.seeker_id = ?1 OR s.provider_id = ?1)",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    HttpResponse::Ok().json(serde_json::json!({"count": count}))
}
