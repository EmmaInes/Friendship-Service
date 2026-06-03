use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use super::extract_user_id;

#[derive(Debug, Deserialize)]
pub struct SurveyRequest {
    pub survey_type: String,
    pub categories: Vec<String>,
    pub budget_min: Option<i64>,
    pub budget_max: Option<i64>,
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
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
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

    let existing = sqlx::query_scalar!(
        "SELECT id FROM surveys WHERE user_id = $1 AND survey_type = $2",
        user_id, body.survey_type
    )
    .fetch_optional(&state.db)
    .await;

    let result = match existing {
        Ok(Some(id)) => {
            sqlx::query!(
                "UPDATE surveys SET categories = $1, budget_min = $2, budget_max = $3,
                 availability = $4, location_preference = $5, experience_level = $6,
                 description = $7, urgency = $8, updated_at = NOW()
                 WHERE id = $9",
                categories_json, body.budget_min, body.budget_max,
                availability, location_preference, experience_level,
                description, urgency, id
            )
            .execute(&state.db)
            .await
        }
        _ => {
            let id = Uuid::new_v4().to_string();
            sqlx::query!(
                "INSERT INTO surveys (id, user_id, survey_type, categories, budget_min, budget_max,
                 availability, location_preference, experience_level, description, urgency)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                id, user_id, body.survey_type, categories_json,
                body.budget_min, body.budget_max, availability,
                location_preference, experience_level, description, urgency
            )
            .execute(&state.db)
            .await
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
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    };

    let rows = sqlx::query!(
        "SELECT id, survey_type, categories, budget_min, budget_max,
                availability, location_preference, experience_level,
                description, urgency, created_at, updated_at
         FROM surveys WHERE user_id = $1",
        user_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(surveys) => {
            let result: Vec<serde_json::Value> = surveys
                .into_iter()
                .map(|r| {
                    let categories: serde_json::Value =
                        serde_json::from_str(&r.categories).unwrap_or(serde_json::json!([]));
                    serde_json::json!({
                        "id": r.id,
                        "survey_type": r.survey_type,
                        "categories": categories,
                        "budget_min": r.budget_min,
                        "budget_max": r.budget_max,
                        "availability": r.availability,
                        "location_preference": r.location_preference,
                        "experience_level": r.experience_level,
                        "description": r.description,
                        "urgency": r.urgency,
                        "created_at": r.created_at,
                        "updated_at": r.updated_at,
                    })
                })
                .collect();
            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            tracing::error!("Failed to fetch surveys: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to fetch surveys"}))
        }
    }
}

pub async fn suggestions(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let user_id = match extract_user_id(&req, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e})),
    };

    let survey = sqlx::query!(
        "SELECT categories, budget_min, budget_max, location_preference
         FROM surveys WHERE user_id = $1 AND survey_type = 'seeker'",
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    let (cats_json, budget_min, budget_max, location_pref) = match survey {
        Ok(Some(s)) => (s.categories, s.budget_min, s.budget_max, s.location_preference),
        _ => return HttpResponse::Ok().json(serde_json::json!({
            "suggestions": [],
            "message": "Complete your seeker survey first to get suggestions"
        })),
    };

    let preferred_categories: Vec<String> =
        serde_json::from_str(&cats_json).unwrap_or_default();

    let rows = sqlx::query!(
        "SELECT s.id, s.provider_id, s.title, s.description, s.category,
                s.price_cents, s.price_type, s.location, s.created_at,
                u.display_name, u.username
         FROM services s
         JOIN users u ON u.id = s.provider_id
         WHERE s.is_active = true AND s.provider_id != $1
         ORDER BY s.created_at DESC",
        user_id
    )
    .fetch_all(&state.db)
    .await;

    let rows = match rows {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to fetch services for suggestions: {}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to fetch suggestions"}));
        }
    };

    let mut suggestions: Vec<serde_json::Value> = rows
        .into_iter()
        .filter_map(|r| {
            let mut score: i32 = 0;
            let mut reasons: Vec<String> = Vec::new();

            if preferred_categories.iter().any(|c| c.eq_ignore_ascii_case(&r.category)) {
                score += 3;
                reasons.push("Matches your preferred category".to_string());
            }

            if let Some(price) = r.price_cents {
                if let Some(max) = budget_max {
                    if price <= max {
                        score += 2;
                        reasons.push("Within your budget".to_string());
                    }
                }
                if let Some(min) = budget_min {
                    if price >= min { score += 1; }
                }
            } else if r.price_type == "free" {
                score += 2;
                reasons.push("Free service".to_string());
            } else if r.price_type == "negotiable" {
                score += 1;
                reasons.push("Price is negotiable".to_string());
            }

            if !location_pref.is_empty() && !r.location.is_empty()
                && r.location.to_lowercase().contains(&location_pref.to_lowercase())
            {
                score += 2;
                reasons.push("Matches your location preference".to_string());
            }

            if score > 0 {
                Some(serde_json::json!({
                    "service": {
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
                    },
                    "score": score,
                    "reasons": reasons,
                }))
            } else {
                None
            }
        })
        .collect();

    suggestions.sort_by(|a, b| {
        b["score"].as_i64().unwrap_or(0).cmp(&a["score"].as_i64().unwrap_or(0))
    });

    HttpResponse::Ok().json(serde_json::json!({"suggestions": suggestions}))
}
