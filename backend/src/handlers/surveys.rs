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
pub struct SurveyRequest {
    pub survey_type: String,           // "provider" or "seeker"
    pub categories: Vec<String>,       // selected categories
    pub budget_min: Option<i64>,       // cents
    pub budget_max: Option<i64>,       // cents
    pub availability: Option<String>,
    pub location_preference: Option<String>,
    pub experience_level: Option<String>,
    pub description: Option<String>,
    pub urgency: Option<String>,
}

pub async fn upsert(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<SurveyRequest>,
) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    if body.survey_type != "provider" && body.survey_type != "seeker" {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "survey_type must be 'provider' or 'seeker'"}));
    }

    let categories_json = serde_json::to_string(&body.categories).unwrap();
    let availability = body.availability.as_deref().unwrap_or("");
    let location_preference = body.location_preference.as_deref().unwrap_or("");
    let experience_level = body.experience_level.as_deref().unwrap_or("");
    let description = body.description.as_deref().unwrap_or("");
    let urgency = body.urgency.as_deref().unwrap_or("flexible");

    let db = state.db.lock().unwrap();

    // Check if survey already exists for this user+type
    let existing = db.query_row(
        "SELECT id FROM surveys WHERE user_id = ?1 AND survey_type = ?2",
        rusqlite::params![user_id, body.survey_type],
        |row| row.get::<_, String>(0),
    );

    let result = match existing {
        Ok(id) => {
            // Update existing
            db.execute(
                "UPDATE surveys SET categories = ?1, budget_min = ?2, budget_max = ?3,
                 availability = ?4, location_preference = ?5, experience_level = ?6,
                 description = ?7, urgency = ?8,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?9",
                rusqlite::params![
                    categories_json, body.budget_min, body.budget_max,
                    availability, location_preference, experience_level,
                    description, urgency, id
                ],
            )
        }
        Err(_) => {
            // Insert new
            let id = Uuid::new_v4().to_string();
            db.execute(
                "INSERT INTO surveys (id, user_id, survey_type, categories, budget_min, budget_max,
                 availability, location_preference, experience_level, description, urgency)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    id, user_id, body.survey_type, categories_json,
                    body.budget_min, body.budget_max, availability,
                    location_preference, experience_level, description, urgency
                ],
            )
        }
    };

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Survey saved"})),
        Err(e) => {
            tracing::error!("Failed to save survey: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to save survey"}))
        }
    }
}

pub async fn get_mine(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let db = state.db.lock().unwrap();

    let mut stmt = db
        .prepare(
            "SELECT id, survey_type, categories, budget_min, budget_max,
                    availability, location_preference, experience_level,
                    description, urgency, created_at, updated_at
             FROM surveys WHERE user_id = ?1",
        )
        .unwrap();

    let surveys: Vec<serde_json::Value> = stmt
        .query_map(rusqlite::params![user_id], |row| {
            let cats_str = row.get::<_, String>(2)?;
            let categories: serde_json::Value =
                serde_json::from_str(&cats_str).unwrap_or(serde_json::json!([]));
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "survey_type": row.get::<_, String>(1)?,
                "categories": categories,
                "budget_min": row.get::<_, Option<i64>>(3)?,
                "budget_max": row.get::<_, Option<i64>>(4)?,
                "availability": row.get::<_, String>(5)?,
                "location_preference": row.get::<_, String>(6)?,
                "experience_level": row.get::<_, String>(7)?,
                "description": row.get::<_, String>(8)?,
                "urgency": row.get::<_, String>(9)?,
                "created_at": row.get::<_, String>(10)?,
                "updated_at": row.get::<_, String>(11)?,
            }))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    HttpResponse::Ok().json(surveys)
}

/// Suggestions: match seeker survey preferences against active services
pub async fn suggestions(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let db = state.db.lock().unwrap();

    // Get seeker survey
    let survey = db.query_row(
        "SELECT categories, budget_min, budget_max, location_preference
         FROM surveys WHERE user_id = ?1 AND survey_type = 'seeker'",
        rusqlite::params![user_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<i64>>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, String>(3)?,
            ))
        },
    );

    let (cats_json, budget_min, budget_max, location_pref) = match survey {
        Ok(s) => s,
        Err(_) => {
            return HttpResponse::Ok().json(serde_json::json!({
                "suggestions": [],
                "message": "Complete your seeker survey first to get suggestions"
            }));
        }
    };

    let preferred_categories: Vec<String> =
        serde_json::from_str(&cats_json).unwrap_or_default();

    // Fetch all active services not owned by this user
    let mut stmt = db
        .prepare(
            "SELECT s.id, s.provider_id, s.title, s.description, s.category,
                    s.price_cents, s.price_type, s.location, s.created_at,
                    u.display_name, u.username
             FROM services s
             JOIN users u ON u.id = s.provider_id
             WHERE s.is_active = 1 AND s.provider_id != ?1
             ORDER BY s.created_at DESC",
        )
        .unwrap();

    let mut suggestions: Vec<serde_json::Value> = Vec::new();

    let rows: Vec<(String, String, String, String, String, Option<i64>, String, String, String, String, String)> = stmt
        .query_map(rusqlite::params![user_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
            ))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    for (id, provider_id, title, description, category, price_cents, price_type, location, created_at, provider_name, provider_username) in rows {
        let mut score: i32 = 0;
        let mut reasons: Vec<String> = Vec::new();

        // Category match
        if preferred_categories.iter().any(|c| c.eq_ignore_ascii_case(&category)) {
            score += 3;
            reasons.push("Matches your preferred category".to_string());
        }

        // Budget match
        if let Some(price) = price_cents {
            if let Some(max) = budget_max {
                if price <= max {
                    score += 2;
                    reasons.push("Within your budget".to_string());
                }
            }
            if let Some(min) = budget_min {
                if price >= min {
                    score += 1;
                }
            }
        } else if price_type == "free" {
            score += 2;
            reasons.push("Free service".to_string());
        } else if price_type == "negotiable" {
            score += 1;
            reasons.push("Price is negotiable".to_string());
        }

        // Location match
        if !location_pref.is_empty() && !location.is_empty()
            && location.to_lowercase().contains(&location_pref.to_lowercase())
        {
            score += 2;
            reasons.push("Matches your location preference".to_string());
        }

        if score > 0 {
            suggestions.push(serde_json::json!({
                "service": {
                    "id": id,
                    "provider_id": provider_id,
                    "title": title,
                    "description": description,
                    "category": category,
                    "price_cents": price_cents,
                    "price_type": price_type,
                    "location": location,
                    "created_at": created_at,
                    "provider_name": provider_name,
                    "provider_username": provider_username,
                },
                "score": score,
                "reasons": reasons,
            }));
        }
    }

    // Sort by score descending
    suggestions.sort_by(|a, b| {
        let sa = a["score"].as_i64().unwrap_or(0);
        let sb = b["score"].as_i64().unwrap_or(0);
        sb.cmp(&sa)
    });

    HttpResponse::Ok().json(serde_json::json!({
        "suggestions": suggestions
    }))
}
